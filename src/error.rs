use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("请求错误: {0}")]
    Request(#[from] reqwest::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("未授权")]
    Unauthorized,

    #[error("Token无效")]
    InvalidToken,

    #[error("Token已过期")]
    TokenExpired,

    #[error("用户不存在")]
    UserNotFound,

    #[error("用户已被禁用")]
    UserDisabled,

    #[error("微信API错误: {0}")]
    WechatApi(String),

    #[error("手机号获取失败")]
    PhoneGetFailed,

    #[error("参数验证失败: {0}")]
    Validation(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误")
            }
            AppError::Request(ref e) => {
                tracing::error!("Request error: {}", e);
                (StatusCode::BAD_GATEWAY, "外部请求失败")
            }
            AppError::Config(ref msg) => {
                tracing::error!("Config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "配置错误")
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "未授权"),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Token无效"),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token已过期"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "用户不存在"),
            AppError::UserDisabled => (StatusCode::FORBIDDEN, "用户已被禁用"),
            AppError::WechatApi(ref msg) => {
                tracing::error!("Wechat API error: {}", msg);
                (StatusCode::BAD_GATEWAY, msg.as_str())
            }
            AppError::PhoneGetFailed => (StatusCode::BAD_REQUEST, "手机号获取失败"),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部错误")
            }
        };

        let body = Json(json!({
            "code": status.as_u16(),
            "message": error_message,
        }));

        (status, body).into_response()
    }
}
