pub mod session;
pub mod user;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};

/// 事务管理 Trait
/// 提供统一的事务处理接口
#[async_trait]
pub trait TransactionManager: Send + Sync {
    /// 在事务中执行操作
    async fn with_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(
                &'a mut Transaction<'static, Postgres>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
        T: Send,
        E: From<sqlx::Error> + Send;
}

/// 数据库连接池的事务管理实现
#[async_trait]
impl TransactionManager for PgPool {
    async fn with_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(
                &'a mut Transaction<'static, Postgres>,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
        T: Send,
        E: From<sqlx::Error> + Send,
    {
        let mut tx = self.begin().await?;
        let result = f(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}
