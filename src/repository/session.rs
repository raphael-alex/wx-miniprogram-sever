use crate::error::{AppError, AppResult};
use crate::model::session::{CreateSession, Session};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::{query_as, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &CreateSession) -> AppResult<Session>;
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Session>>;
    async fn delete_by_user_id(&self, user_id: Uuid) -> AppResult<()>;
}

/// 会话仓库实现
pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建或更新会话 (先删除旧的再创建新的)
    pub async fn upsert(&self, session: &CreateSession) -> AppResult<Session> {
        // 先删除该用户的旧会话
        sqlx::query(
            "DELETE FROM sessions WHERE user_id = $1",
        )
        .bind(session.user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        // 创建新会话
        self.create(session).await
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create(&self, session: &CreateSession) -> AppResult<Session> {
        query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, session_key, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, session_key, expires_at, created_at
            "#,
        )
        .bind(session.user_id)
        .bind(&session.session_key)
        .bind(session.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Session>> {
        query_as::<_, Session>(
            r#"
            SELECT id, user_id, session_key, expires_at, created_at
            FROM sessions
            WHERE user_id = $1 AND expires_at > NOW()
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    async fn delete_by_user_id(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM sessions WHERE user_id = $1",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}

// ============ 事务版本的会话操作 ============

pub struct SessionTxRepository;

impl SessionTxRepository {
    /// 在事务中删除旧会话并创建新会话
    pub async fn upsert_in_tx(
        tx: &mut Transaction<'static, Postgres>,
        session: &CreateSession,
    ) -> AppResult<Session> {
        // 删除旧会话
        sqlx::query(
            "DELETE FROM sessions WHERE user_id = $1",
        )
        .bind(session.user_id)
        .execute(tx.as_mut())
        .await
        .map_err(AppError::Database)?;

        // 创建新会话
        query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, session_key, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, session_key, expires_at, created_at
            "#,
        )
        .bind(session.user_id)
        .bind(&session.session_key)
        .bind(session.expires_at)
        .fetch_one(tx.as_mut())
        .await
        .map_err(AppError::Database)
    }

    /// 创建会话 (默认30天过期)
    pub fn create_session_data(user_id: Uuid, session_key: String) -> CreateSession {
        CreateSession {
            user_id,
            session_key,
            expires_at: Utc::now() + Duration::days(30),
        }
    }
}
