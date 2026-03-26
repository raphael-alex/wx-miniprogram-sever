use crate::error::AppResult;
use crate::model::request::{UpdateProfileRequest, UserResponse};
use crate::model::user::UpdateUser;
use crate::service::AuthService;
use crate::utils::jwt::Claims;
use axum::{
    extract::State,
    Extension, Json,
};
use std::sync::Arc;

/// 获取用户信息
pub async fn get_profile(
    State(auth_service): State<Arc<AuthService>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    let user = auth_service.get_user(claims.sub).await?;

    Ok(Json(UserResponse {
        id: user.id,
        phone: user.phone,
        nickname: user.nickname,
        avatar_url: user.avatar_url,
        gender: user.gender,
        status: user.status,
    }))
}

/// 更新用户信息
pub async fn update_profile(
    State(auth_service): State<Arc<AuthService>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserResponse>> {
    let update = UpdateUser {
        nickname: req.nickname,
        avatar_url: req.avatar_url,
        gender: req.gender,
    };

    let user = auth_service.update_user(claims.sub, update).await?;

    Ok(Json(UserResponse {
        id: user.id,
        phone: user.phone,
        nickname: user.nickname,
        avatar_url: user.avatar_url,
        gender: user.gender,
        status: user.status,
    }))
}
