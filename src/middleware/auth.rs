use crate::utils::jwt::JwtService;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

/// JWT认证中间件
pub async fn auth_layer(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = auth_header
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = JwtService::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 将用户信息注入到请求扩展中
    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
