use crate::presentation::handlers::user_handlers::{
    create_user, delete_user, get_user, list_users, update_user, UserHandlerState,
};
use crate::presentation::handlers::auth_handlers::{login, AuthState};
use crate::presentation::middleware::auth_middleware;
use crate::infrastructure::security::JwtManager;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn user_routes(
    user_state: Arc<UserHandlerState>,
    auth_state: Arc<AuthState>,
    jwt_manager: Arc<JwtManager>,
) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/auth/login", post(login))
        .with_state(auth_state)
        .route("/users", post(create_user))
        .with_state(user_state.clone());

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", axum::routing::put(update_user))
        .route("/users/:id", axum::routing::delete(delete_user))
        .with_state(user_state)
        .layer(middleware::from_fn_with_state(
            jwt_manager,
            auth_middleware,
        ));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}
