use crate::error::{AppError, AppResult};
use crate::model::user::{CreateUser, UpdateUser, User};
use async_trait::async_trait;
use sqlx::{query_as, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<User>;
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>>;
    async fn create(&self, user: &CreateUser) -> AppResult<User>;
    async fn update(&self, id: Uuid, user: &UpdateUser) -> AppResult<User>;
    async fn update_phone(&self, id: Uuid, phone: &str) -> AppResult<()>;
}

/// 用户仓库实现
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<User> {
        query_as::<_, User>(
            r#"
            SELECT id, openid, unionid, phone, nickname, avatar_url, gender, status, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::UserNotFound,
            other => AppError::Database(other),
        })
    }

    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        query_as::<_, User>(
            r#"
            SELECT id, openid, unionid, phone, nickname, avatar_url, gender, status, created_at, updated_at
            FROM users
            WHERE openid = $1
            "#,
        )
        .bind(openid)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    async fn create(&self, user: &CreateUser) -> AppResult<User> {
        query_as::<_, User>(
            r#"
            INSERT INTO users (openid, unionid)
            VALUES ($1, $2)
            RETURNING id, openid, unionid, phone, nickname, avatar_url, gender, status, created_at, updated_at
            "#,
        )
        .bind(&user.openid)
        .bind(&user.unionid)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    async fn update(&self, id: Uuid, user: &UpdateUser) -> AppResult<User> {
        query_as::<_, User>(
            r#"
            UPDATE users
            SET 
                nickname = COALESCE($2, nickname),
                avatar_url = COALESCE($3, avatar_url),
                gender = COALESCE($4, gender)
            WHERE id = $1
            RETURNING id, openid, unionid, phone, nickname, avatar_url, gender, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&user.nickname)
        .bind(&user.avatar_url)
        .bind(user.gender)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::UserNotFound,
            other => AppError::Database(other),
        })
    }

    async fn update_phone(&self, id: Uuid, phone: &str) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET phone = $2 WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(phone)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }
}

// ============ 事务版本的用户仓库操作 ============

/// 事务内的用户操作
pub struct UserTxRepository;

impl UserTxRepository {
    /// 在事务中创建用户
    pub async fn create_in_tx(
        tx: &mut Transaction<'static, Postgres>,
        user: &CreateUser,
    ) -> AppResult<User> {
        query_as::<_, User>(
            r#"
            INSERT INTO users (openid, unionid)
            VALUES ($1, $2)
            RETURNING id, openid, unionid, phone, nickname, avatar_url, gender, status, created_at, updated_at
            "#,
        )
        .bind(&user.openid)
        .bind(&user.unionid)
        .fetch_one(tx.as_mut())
        .await
        .map_err(AppError::Database)
    }

    /// 在事务中更新手机号
    pub async fn update_phone_in_tx(
        tx: &mut Transaction<'static, Postgres>,
        id: Uuid,
        phone: &str,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET phone = $2 WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(phone)
        .execute(tx.as_mut())
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }

    /// 在事务中查找用户
    pub async fn find_by_openid_in_tx(
        tx: &mut Transaction<'static, Postgres>,
        openid: &str,
    ) -> AppResult<Option<User>> {
        query_as::<_, User>(
            r#"
            SELECT id, openid, unionid, phone, nickname, avatar_url, gender, status, created_at, updated_at
            FROM users
            WHERE openid = $1
            "#,
        )
        .bind(openid)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(AppError::Database)
    }
}
