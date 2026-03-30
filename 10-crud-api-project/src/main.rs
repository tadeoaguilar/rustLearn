mod application;
mod domain;
mod infrastructure;
mod presentation;

use crate::application::commands::{CreateUserCommand, DeleteUserCommand, UpdateUserCommand};
use crate::application::queries::{GetUserQuery, ListUsersQuery};
use crate::infrastructure::config::Config;
use crate::infrastructure::database::PostgresUserRepository;
use crate::infrastructure::security::{JwtManager, PasswordHasher};
use crate::presentation::handlers::auth_handlers::AuthState;
use crate::presentation::handlers::user_handlers::UserHandlerState;
use crate::presentation::routes::user_routes;
use anyhow::{Context, Result};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().context("Failed to load configuration")?;
    tracing::info!("Configuration loaded successfully");

    // Setup database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;
    tracing::info!("Database connection established");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .context("Failed to run database migrations")?;
    tracing::info!("Database migrations completed");

    // ========================================
    // DEPENDENCY INJECTION SETUP
    // ========================================

    // Infrastructure layer instances
    let user_repository = Arc::new(PostgresUserRepository::new(db_pool.clone()))
        as Arc<dyn domain::repositories::UserRepository>;
    let password_hasher = Arc::new(PasswordHasher::new());
    let jwt_manager = Arc::new(JwtManager::new(
        config.jwt_secret.clone(),
        config.jwt_expiration_hours,
    ));

    // Application layer - Commands (writes)
    let create_user_command = Arc::new(CreateUserCommand::new(
        user_repository.clone(),
        password_hasher.clone(),
    ));
    let update_user_command = Arc::new(UpdateUserCommand::new(user_repository.clone()));
    let delete_user_command = Arc::new(DeleteUserCommand::new(user_repository.clone()));

    // Application layer - Queries (reads)
    let get_user_query = Arc::new(GetUserQuery::new(user_repository.clone()));
    let list_users_query = Arc::new(ListUsersQuery::new(user_repository.clone()));

    // Presentation layer - Handler states (dependency containers)
    let user_handler_state = Arc::new(UserHandlerState {
        create_user_command,
        update_user_command,
        delete_user_command,
        get_user_query,
        list_users_query,
    });

    let auth_state = Arc::new(AuthState {
        user_repository: user_repository.clone(),
        jwt_manager: jwt_manager.clone(),
        password_hasher: password_hasher.clone(),
    });

    // ========================================
    // BUILD APPLICATION ROUTES
    // ========================================

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", user_routes(user_handler_state, auth_state, jwt_manager))
        .layer(cors);

    // Start server
    let listener = tokio::net::TcpListener::bind(config.server_address())
        .await
        .context("Failed to bind server address")?;

    tracing::info!("Server running on http://{}", listener.local_addr()?);
    tracing::info!("API available at http://{}/api", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
