use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,      // user_id
    pub openid: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtService;

impl JwtService {
    pub fn generate_token(user_id: Uuid, openid: &str) -> AppResult<String> {
        let config = AppConfig::get();
        let now = Utc::now();
        let exp = now + Duration::seconds(config.jwt.expires_in);

        let claims = Claims {
            sub: user_id,
            openid: openid.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Token生成失败: {}", e)))
    }

    pub fn verify_token(token: &str) -> AppResult<Claims> {
        let config = AppConfig::get();

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("ExpiredSignature") {
                AppError::TokenExpired
            } else {
                AppError::InvalidToken
            }
        })?;

        Ok(token_data.claims)
    }
}
