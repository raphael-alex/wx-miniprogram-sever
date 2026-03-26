use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use wx_miniprogram_server::{
    api::{auth, user},
    config::AppConfig,
    repository::{
        session::PgSessionRepository,
        user::PgUserRepository,
    },
    service::{AuthService, WechatService},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug,sqlx=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = AppConfig::init();
    tracing::info!("Loaded configuration");

    // 连接数据库
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database.url)
        .await?;
    tracing::info!("Database connected");

    // 初始化仓库
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let session_repo = Arc::new(PgSessionRepository::new(pool.clone()));
    let wechat_service = Arc::new(WechatService::new());

    // 初始化服务
    let auth_service = Arc::new(AuthService::new(
        pool.clone(),
        user_repo,
        session_repo,
        wechat_service,
    ));

    // 构建路由
    let app = Router::new()
        // 公开路由 (无需认证)
        .route("/api/auth/login", post(auth::login))
        // 受保护路由 (需要JWT认证)
        .nest(
            "/api",
            Router::new()
                .route("/auth/phone", post(auth::bind_phone))
                .route("/user/profile", get(user::get_profile).put(user::update_profile))
                .layer(middleware::from_fn(wx_miniprogram_server::middleware::auth_layer)),
        )
        .with_state(auth_service)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
