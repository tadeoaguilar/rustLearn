use crate::infrastructure::security::jwt::{Claims, JwtManager};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
}

pub async fn auth_middleware(
    State(jwt_manager): State<Arc<JwtManager>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(h.trim_start_matches("Bearer "))
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token
    let claims = jwt_manager
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user info to request extensions
    let auth_user = AuthUser {
        user_id: claims.sub,
        username: claims.username,
    };
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
