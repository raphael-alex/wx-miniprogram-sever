use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============ 请求体 ============

/// 微信登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub code: String,
}

/// 获取手机号请求
#[derive(Debug, Deserialize)]
pub struct PhoneRequest {
    pub code: String,
}

/// 更新用户信息请求
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<i16>,
}

// ============ 响应体 ============

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

/// 用户信息响应
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub phone: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: i16,
    pub status: i16,
}

/// 手机号响应
#[derive(Debug, Serialize)]
pub struct PhoneResponse {
    pub phone: String,
}

/// 通用响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: u16, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
}

// ============ 微信API响应 ============

/// 微信code2Session响应
#[derive(Debug, Deserialize)]
pub struct WechatCode2SessionResponse {
    pub openid: Option<String>,
    pub session_key: Option<String>,
    pub unionid: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

/// 微信获取手机号响应
#[derive(Debug, Deserialize)]
pub struct WechatPhoneNumberResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub phone_info: Option<WechatPhoneInfo>,
}

#[derive(Debug, Deserialize)]
pub struct WechatPhoneInfo {
    pub phone_number: Option<String>,
    pub pure_phone_number: Option<String>,
    pub country_code: Option<String>,
    pub watermark: Option<WechatWatermark>,
}

#[derive(Debug, Deserialize)]
pub struct WechatWatermark {
    pub appid: String,
    pub timestamp: i64,
}

/// 微信AccessToken响应
#[derive(Debug, Deserialize)]
pub struct WechatAccessTokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<i32>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}
