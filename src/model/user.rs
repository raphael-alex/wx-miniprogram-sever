use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub openid: String,
    pub unionid: Option<String>,
    pub phone: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: i16,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUser {
    pub openid: String,
    pub unionid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateUser {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<i16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub phone: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: i16,
    pub status: i16,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            phone: user.phone,
            nickname: user.nickname,
            avatar_url: user.avatar_url,
            gender: user.gender,
            status: user.status,
            created_at: user.created_at,
        }
    }
}
