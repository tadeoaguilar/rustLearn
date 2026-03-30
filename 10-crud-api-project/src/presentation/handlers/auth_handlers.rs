use crate::application::dtos::{AuthResponseDto, LoginDto, UserDto};
use crate::domain::repositories::UserRepository;
use crate::infrastructure::security::{JwtManager, PasswordHasher};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub struct AuthState {
    pub user_repository: Arc<dyn UserRepository>,
    pub jwt_manager: Arc<JwtManager>,
    pub password_hasher: Arc<PasswordHasher>,
}

pub async fn login(
    State(state): State<Arc<AuthState>>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<AuthResponseDto>, (StatusCode, String)> {
    // Find user by username
    let user = state
        .user_repository
        .find_by_username(&dto.username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid credentials".to_string(),
            )
        })?;

    // Verify password
    let is_valid = state
        .password_hasher
        .verify_password(&dto.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid credentials".to_string(),
        ));
    }

    // Check if user is active
    if !user.is_active {
        return Err((StatusCode::FORBIDDEN, "Account is inactive".to_string()));
    }

    // Generate JWT token
    let token = state
        .jwt_manager
        .generate_token(user.id, &user.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Create response
    let full_name = user.full_name();
    let user_dto = UserDto {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        full_name,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    };

    Ok(Json(AuthResponseDto {
        token,
        user: user_dto,
    }))
}
