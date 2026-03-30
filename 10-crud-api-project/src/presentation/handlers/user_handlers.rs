use crate::application::commands::{CreateUserCommand, DeleteUserCommand, UpdateUserCommand};
use crate::application::dtos::{CreateUserDto, PaginatedResponse, UpdateUserDto, UserDto};
use crate::application::queries::{GetUserQuery, ListUsersQuery};
use crate::presentation::middleware::AuthUser;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

// Dependency container for user handlers
pub struct UserHandlerState {
    pub create_user_command: Arc<CreateUserCommand>,
    pub update_user_command: Arc<UpdateUserCommand>,
    pub delete_user_command: Arc<DeleteUserCommand>,
    pub get_user_query: Arc<GetUserQuery>,
    pub list_users_query: Arc<ListUsersQuery>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

// POST /api/users - Create a new user (public endpoint)
pub async fn create_user(
    State(state): State<Arc<UserHandlerState>>,
    Json(dto): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserDto>), (StatusCode, String)> {
    let user = state
        .create_user_command
        .execute(dto)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(user)))
}

// GET /api/users - List all users (protected)
pub async fn list_users(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<Arc<UserHandlerState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<UserDto>>, (StatusCode, String)> {
    let users = state
        .list_users_query
        .execute(params.page, params.page_size)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

// GET /api/users/:id - Get user by ID (protected)
pub async fn get_user(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<Arc<UserHandlerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserDto>, (StatusCode, String)> {
    let user = state
        .get_user_query
        .execute(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(user))
}

// PUT /api/users/:id - Update user (protected)
pub async fn update_user(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<Arc<UserHandlerState>>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<UserDto>, (StatusCode, String)> {
    let user = state
        .update_user_command
        .execute(id, dto)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(user))
}

// DELETE /api/users/:id - Delete user (protected)
pub async fn delete_user(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<Arc<UserHandlerState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .delete_user_command
        .execute(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
