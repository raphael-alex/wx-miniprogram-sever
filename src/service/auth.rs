use crate::error::{AppError, AppResult};
use crate::model::session::CreateSession;
use crate::model::user::{CreateUser, UpdateUser, User};
use crate::repository::{
    session::SessionTxRepository,
    user::{UserRepository, UserTxRepository},
    TransactionManager,
};
use crate::service::wechat::WechatService;
use crate::utils::jwt::JwtService;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    pool: PgPool,
    user_repo: Arc<dyn UserRepository>,
    wechat_service: Arc<WechatService>,
}

impl AuthService {
    pub fn new(
        pool: PgPool,
        user_repo: Arc<dyn UserRepository>,
        _session_repo: Arc<dyn crate::repository::session::SessionRepository>,
        wechat_service: Arc<WechatService>,
    ) -> Self {
        Self {
            pool,
            user_repo,
            wechat_service,
        }
    }

    /// 微信登录
    pub async fn login(&self, code: &str) -> AppResult<(User, String)> {
        // 1. 调用微信API获取openid和session_key
        let wechat_resp = self.wechat_service.code2session(code).await?;

        let openid = wechat_resp.openid.unwrap();
        let session_key = wechat_resp.session_key.unwrap();

        // 2. 使用事务处理登录逻辑
        let user = self
            .pool
            .with_transaction(|tx| {
                Box::pin(async move {
                    // 查找或创建用户
                    let user = match UserTxRepository::find_by_openid_in_tx(tx, &openid).await? {
                        Some(u) => u,
                        None => {
                            let new_user = CreateUser {
                                openid: openid.clone(),
                                unionid: wechat_resp.unionid,
                            };
                            UserTxRepository::create_in_tx(tx, &new_user).await?
                        }
                    };

                    // 检查用户状态
                    if user.status == 0 {
                        return Err(AppError::UserDisabled);
                    }

                    // 创建会话
                    let session_data = CreateSession {
                        user_id: user.id,
                        session_key,
                        expires_at: Utc::now() + Duration::days(30),
                    };
                    SessionTxRepository::upsert_in_tx(tx, &session_data).await?;

                    Ok(user)
                })
            })
            .await?;

        // 3. 生成JWT token
        let token = JwtService::generate_token(user.id, &user.openid)?;

        Ok((user, token))
    }

    /// 获取并绑定手机号
    pub async fn bind_phone(&self, user_id: Uuid, code: &str) -> AppResult<String> {
        // 1. 调用微信API获取手机号
        let phone = self.wechat_service.get_phone_number(code).await?;

        // 2. 更新用户手机号
        self.user_repo.update_phone(user_id, &phone).await?;

        Ok(phone)
    }

    /// 获取用户信息
    pub async fn get_user(&self, user_id: Uuid) -> AppResult<User> {
        let user = self.user_repo.find_by_id(user_id).await?;

        if user.status == 0 {
            return Err(AppError::UserDisabled);
        }

        Ok(user)
    }

    /// 更新用户信息
    pub async fn update_user(&self, user_id: Uuid, update: UpdateUser) -> AppResult<User> {
        self.user_repo.update(user_id, &update).await
    }
}
