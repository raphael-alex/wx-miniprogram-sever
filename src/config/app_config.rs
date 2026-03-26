use once_cell::sync::OnceCell;
use std::env;

static CONFIG: OnceCell<AppConfig> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub wechat: WechatConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone)]
pub struct WechatConfig {
    pub appid: String,
    pub secret: String,
}

impl AppConfig {
    pub fn init() -> Self {
        dotenvy::dotenv().ok();

        let config = Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .expect("SERVER_PORT must be a valid port number"),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set"),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "default-secret-key".to_string()),
                expires_in: env::var("JWT_EXPIRES_IN")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()
                    .expect("JWT_EXPIRES_IN must be a valid number"),
            },
            wechat: WechatConfig {
                appid: env::var("WECHAT_APPID")
                    .expect("WECHAT_APPID must be set"),
                secret: env::var("WECHAT_SECRET")
                    .expect("WECHAT_SECRET must be set"),
            },
        };

        CONFIG.set(config.clone()).ok();
        config
    }

    pub fn get() -> &'static Self {
        CONFIG.get().expect("Config not initialized")
    }
}
