use crate::error::AppResult;
use crate::model::request::{LoginRequest, LoginResponse, PhoneRequest, PhoneResponse, UserResponse};
use crate::service::AuthService;
use crate::utils::jwt::Claims;
use axum::{
    extract::State,
    Extension, Json,
};
use std::sync::Arc;

/// 微信登录
pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let (user, token) = auth_service.login(&req.code).await?;

    Ok(Json(LoginResponse {
        token,
        user: UserResponse {
            id: user.id,
            phone: user.phone,
            nickname: user.nickname,
            avatar_url: user.avatar_url,
            gender: user.gender,
            status: user.status,
        },
    }))
}

/// 获取并绑定手机号
pub async fn bind_phone(
    State(auth_service): State<Arc<AuthService>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PhoneRequest>,
) -> AppResult<Json<PhoneResponse>> {
    let phone = auth_service.bind_phone(claims.sub, &req.code).await?;

    Ok(Json(PhoneResponse { phone }))
}
