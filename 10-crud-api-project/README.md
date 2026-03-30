# Practice 10: Building a CRUD API with Clean Architecture, CQRS, Security & DI in Rust

## Table of Contents
1. [Introduction](#introduction)
2. [Concepts Overview](#concepts-overview)
3. [Architecture](#architecture)
4. [Prerequisites](#prerequisites)
5. [Project Setup](#project-setup)
6. [Step-by-Step Implementation](#step-by-step-implementation)
7. [Testing the API](#testing-the-api)
8. [Running the Application](#running-the-application)

---

## Introduction

This practice will guide you through building a production-ready CRUD (Create, Read, Update, Delete) API in Rust using industry-standard architectural patterns:

- **Clean Architecture**: Separation of concerns with clear layer boundaries
- **CQRS (Command Query Responsibility Segregation)**: Separate read and write operations
- **Dependency Injection**: Loosely coupled components for testability
- **Security**: JWT-based authentication and password hashing
- **Async/Await**: Modern asynchronous programming with Tokio

You'll build a User Management API with full authentication capabilities.

---

## Concepts Overview

### Clean Architecture

Clean Architecture divides the application into layers, each with specific responsibilities:

1. **Domain Layer** (innermost): Core business logic, entities, and repository traits
   - No dependencies on other layers
   - Pure business rules

2. **Application Layer**: Use cases (commands & queries), DTOs
   - Depends only on Domain
   - Orchestrates business logic

3. **Infrastructure Layer**: Concrete implementations (database, external services)
   - Implements Domain interfaces
   - Technical details

4. **Presentation Layer** (outermost): HTTP handlers, routes, middleware
   - Depends on Application layer
   - User interface concerns

**Benefits**: Testability, maintainability, independence from frameworks and databases.

### CQRS (Command Query Responsibility Segregation)

CQRS separates operations into two categories:

- **Commands**: Modify state (Create, Update, Delete) - return success/failure
- **Queries**: Read state (Get, List) - return data without modification

**Benefits**: Optimized read/write models, better scalability, clear intent.

### Dependency Injection in Rust

Unlike OOP languages with DI containers, Rust uses:
- **Traits**: Define abstractions (interfaces)
- **Generic Types**: Accept any implementation of a trait
- **Arc (Atomic Reference Counting)**: Share state across async tasks
- **Type System**: Compile-time dependency resolution

---

## Architecture

### Project Structure

```
10-crud-api-project/
├── Cargo.toml
├── .env
├── .env.example
├── README.md
├── src/
│   ├── main.rs                          # Entry point, DI setup
│   │
│   ├── domain/                          # Domain Layer (Core Business Logic)
│   │   ├── mod.rs
│   │   ├── entities/
│   │   │   ├── mod.rs
│   │   │   └── user.rs                  # User entity
│   │   └── repositories/
│   │       ├── mod.rs
│   │       └── user_repository.rs       # Repository trait (abstraction)
│   │
│   ├── application/                     # Application Layer (Use Cases)
│   │   ├── mod.rs
│   │   ├── commands/                    # Write operations (CQRS)
│   │   │   ├── mod.rs
│   │   │   ├── create_user.rs
│   │   │   ├── update_user.rs
│   │   │   └── delete_user.rs
│   │   ├── queries/                     # Read operations (CQRS)
│   │   │   ├── mod.rs
│   │   │   ├── get_user.rs
│   │   │   └── list_users.rs
│   │   └── dtos/                        # Data Transfer Objects
│   │       ├── mod.rs
│   │       └── user_dto.rs
│   │
│   ├── infrastructure/                  # Infrastructure Layer (Technical Details)
│   │   ├── mod.rs
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   └── user_repository_impl.rs  # PostgreSQL implementation
│   │   ├── security/
│   │   │   ├── mod.rs
│   │   │   ├── jwt.rs                   # JWT token management
│   │   │   └── password.rs              # Password hashing
│   │   └── config.rs                    # Configuration management
│   │
│   └── presentation/                    # Presentation Layer (HTTP API)
│       ├── mod.rs
│       ├── routes/
│       │   ├── mod.rs
│       │   └── user_routes.rs           # Route definitions
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── user_handlers.rs         # HTTP request handlers
│       │   └── auth_handlers.rs         # Authentication handlers
│       └── middleware/
│           ├── mod.rs
│           └── auth_middleware.rs       # JWT validation middleware
│
├── migrations/
│   └── 001_create_users_table.sql       # Database schema
│
└── tests/
    └── integration_tests.rs              # API integration tests
```

### Data Flow Example

**Creating a User (Command)**:
```
HTTP POST /api/users
  ↓
Handler (Presentation) → Validates request
  ↓
CreateUserCommand (Application) → Business logic
  ↓
UserRepository trait (Domain) → Interface
  ↓
UserRepositoryImpl (Infrastructure) → Database
  ↓
PostgreSQL
```

---

## Prerequisites

### Required Software

1. **Rust** (1.75+)
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Verify installation
   rustc --version
   cargo --version
   ```

2. **PostgreSQL** (14+)

   **macOS**:
   ```bash
   brew install postgresql@14
   brew services start postgresql@14
   ```

   **Linux (Ubuntu/Debian)**:
   ```bash
   sudo apt update
   sudo apt install postgresql postgresql-contrib
   sudo systemctl start postgresql
   ```

   **Windows**:
   Download from https://www.postgresql.org/download/windows/

3. **SQLx CLI** (for database migrations)
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

4. **Optional: PostgreSQL Client**
   ```bash
   # macOS
   brew install libpq

   # Linux
   sudo apt install postgresql-client
   ```

---

## Project Setup

### Step 1: Create Database

```bash
# Connect to PostgreSQL
psql postgres

# Create database and user
CREATE DATABASE crud_api_db;
CREATE USER crud_api_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE crud_api_db TO crud_api_user;

# Exit psql
\q
```

### Step 2: Create Project Files

Navigate to the `10-crud-api-project` directory:
```bash
cd 10-crud-api-project
```

All files will be created in the subsequent steps.

---

## Step-by-Step Implementation

### Step 3: Create Cargo.toml

Create the `Cargo.toml` file with all required dependencies:

```toml
[package]
name = "crud-api-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web Framework
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1.35", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Security
jsonwebtoken = "9.2"
bcrypt = "0.15"

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenv = "0.15"

# Validation
validator = { version = "0.16", features = ["derive"] }

[dev-dependencies]
reqwest = { version = "0.11", features = ["json"] }
```

### Step 4: Create Environment Configuration

Create `.env.example`:

```bash
DATABASE_URL=postgresql://crud_api_user:your_secure_password@localhost/crud_api_db
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION_HOURS=24
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RUST_LOG=debug
```

Copy to `.env`:
```bash
cp .env.example .env
```

**Important**: Update the values in `.env` with your actual database credentials.

---

### Step 5: Create Database Migration

Create `migrations/001_create_users_table.sql`:

```sql
-- Create users table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index for faster lookups
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

Run the migration:
```bash
sqlx migrate run
```

---

### Step 6: Domain Layer - Entities

Create `src/domain/entities/user.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: String,
        username: String,
        password_hash: String,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            username,
            password_hash,
            first_name,
            last_name,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }
}
```

Create `src/domain/entities/mod.rs`:

```rust
pub mod user;
pub use user::User;
```

---

### Step 7: Domain Layer - Repository Trait

Create `src/domain/repositories/user_repository.rs`:

```rust
use crate::domain::entities::User;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait defines the contract for data access
/// This is our abstraction - the "interface" in Clean Architecture
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<User>>;
    async fn update(&self, user: User) -> Result<User>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
    async fn count(&self) -> Result<i64>;
}
```

Add to `Cargo.toml` dependencies:
```toml
async-trait = "0.1"
```

Create `src/domain/repositories/mod.rs`:

```rust
pub mod user_repository;
pub use user_repository::UserRepository;
```

Create `src/domain/mod.rs`:

```rust
pub mod entities;
pub mod repositories;
```

---

### Step 8: Application Layer - DTOs

Create `src/application/dtos/user_dto.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 3, max = 100, message = "Username must be between 3 and 100 characters"))]
    pub username: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(max = 100))]
    pub first_name: Option<String>,

    #[validate(length(max = 100))]
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(length(max = 100))]
    pub first_name: Option<String>,

    #[validate(length(max = 100))]
    pub last_name: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(length(min = 1))]
    pub username: String,

    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl PaginatedResponse<UserDto> {
    pub fn new(data: Vec<UserDto>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = (total as f64 / page_size as f64).ceil() as i64;
        Self {
            data,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}
```

Create `src/application/dtos/mod.rs`:

```rust
pub mod user_dto;
pub use user_dto::*;
```

---

### Step 9: Application Layer - Commands (CQRS Write Operations)

Create `src/application/commands/create_user.rs`:

```rust
use crate::application::dtos::{CreateUserDto, UserDto};
use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::security::password::PasswordHasher;
use anyhow::{Context, Result};
use std::sync::Arc;
use validator::Validate;

pub struct CreateUserCommand {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<PasswordHasher>,
}

impl CreateUserCommand {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<PasswordHasher>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
        }
    }

    pub async fn execute(&self, dto: CreateUserDto) -> Result<UserDto> {
        // Validate input
        dto.validate()
            .context("Validation failed for create user")?;

        // Check if email already exists
        if let Some(_) = self
            .user_repository
            .find_by_email(&dto.email)
            .await
            .context("Failed to check email existence")?
        {
            anyhow::bail!("Email already exists");
        }

        // Check if username already exists
        if let Some(_) = self
            .user_repository
            .find_by_username(&dto.username)
            .await
            .context("Failed to check username existence")?
        {
            anyhow::bail!("Username already exists");
        }

        // Hash password
        let password_hash = self
            .password_hasher
            .hash_password(&dto.password)
            .context("Failed to hash password")?;

        // Create user entity
        let user = User::new(
            dto.email,
            dto.username,
            password_hash,
            dto.first_name,
            dto.last_name,
        );

        // Save to repository
        let created_user = self
            .user_repository
            .create(user)
            .await
            .context("Failed to create user in database")?;

        // Convert to DTO
        Ok(user_to_dto(created_user))
    }
}

fn user_to_dto(user: User) -> UserDto {
    let full_name = user.full_name();
    UserDto {
        id: user.id,
        email: user.email,
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        full_name,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}
```

Create `src/application/commands/update_user.rs`:

```rust
use crate::application::dtos::{UpdateUserDto, UserDto};
use crate::domain::repositories::UserRepository;
use anyhow::{Context, Result};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct UpdateUserCommand {
    user_repository: Arc<dyn UserRepository>,
}

impl UpdateUserCommand {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, id: Uuid, dto: UpdateUserDto) -> Result<UserDto> {
        // Validate input
        dto.validate()
            .context("Validation failed for update user")?;

        // Find existing user
        let mut user = self
            .user_repository
            .find_by_id(id)
            .await
            .context("Failed to fetch user")?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Update fields if provided
        if let Some(email) = dto.email {
            // Check if email is already taken by another user
            if let Some(existing) = self.user_repository.find_by_email(&email).await? {
                if existing.id != user.id {
                    anyhow::bail!("Email already exists");
                }
            }
            user.email = email;
        }

        if let Some(first_name) = dto.first_name {
            user.first_name = Some(first_name);
        }

        if let Some(last_name) = dto.last_name {
            user.last_name = Some(last_name);
        }

        if let Some(is_active) = dto.is_active {
            user.is_active = is_active;
        }

        // Save updated user
        let updated_user = self
            .user_repository
            .update(user)
            .await
            .context("Failed to update user in database")?;

        // Convert to DTO
        let full_name = updated_user.full_name();
        Ok(UserDto {
            id: updated_user.id,
            email: updated_user.email,
            username: updated_user.username,
            first_name: updated_user.first_name,
            last_name: updated_user.last_name,
            full_name,
            is_active: updated_user.is_active,
            created_at: updated_user.created_at,
            updated_at: updated_user.updated_at,
        })
    }
}
```

Create `src/application/commands/delete_user.rs`:

```rust
use crate::domain::repositories::UserRepository;
use anyhow::{Context, Result};
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteUserCommand {
    user_repository: Arc<dyn UserRepository>,
}

impl DeleteUserCommand {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, id: Uuid) -> Result<()> {
        // Check if user exists
        let user = self
            .user_repository
            .find_by_id(id)
            .await
            .context("Failed to fetch user")?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Delete user
        let deleted = self
            .user_repository
            .delete(user.id)
            .await
            .context("Failed to delete user from database")?;

        if !deleted {
            anyhow::bail!("Failed to delete user");
        }

        Ok(())
    }
}
```

Create `src/application/commands/mod.rs`:

```rust
pub mod create_user;
pub mod update_user;
pub mod delete_user;

pub use create_user::CreateUserCommand;
pub use update_user::UpdateUserCommand;
pub use delete_user::DeleteUserCommand;
```

---

### Step 10: Application Layer - Queries (CQRS Read Operations)

Create `src/application/queries/get_user.rs`:

```rust
use crate::application::dtos::UserDto;
use crate::domain::repositories::UserRepository;
use anyhow::{Context, Result};
use std::sync::Arc;
use uuid::Uuid;

pub struct GetUserQuery {
    user_repository: Arc<dyn UserRepository>,
}

impl GetUserQuery {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, id: Uuid) -> Result<UserDto> {
        let user = self
            .user_repository
            .find_by_id(id)
            .await
            .context("Failed to fetch user")?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let full_name = user.full_name();
        Ok(UserDto {
            id: user.id,
            email: user.email,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
            full_name,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    pub async fn by_username(&self, username: &str) -> Result<UserDto> {
        let user = self
            .user_repository
            .find_by_username(username)
            .await
            .context("Failed to fetch user")?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let full_name = user.full_name();
        Ok(UserDto {
            id: user.id,
            email: user.email,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
            full_name,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }
}
```

Create `src/application/queries/list_users.rs`:

```rust
use crate::application::dtos::{PaginatedResponse, UserDto};
use crate::domain::repositories::UserRepository;
use anyhow::{Context, Result};
use std::sync::Arc;

pub struct ListUsersQuery {
    user_repository: Arc<dyn UserRepository>,
}

impl ListUsersQuery {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, page: i64, page_size: i64) -> Result<PaginatedResponse<UserDto>> {
        // Calculate offset
        let offset = (page - 1) * page_size;

        // Fetch users
        let users = self
            .user_repository
            .find_all(page_size, offset)
            .await
            .context("Failed to fetch users")?;

        // Get total count
        let total = self
            .user_repository
            .count()
            .await
            .context("Failed to count users")?;

        // Convert to DTOs
        let user_dtos: Vec<UserDto> = users
            .into_iter()
            .map(|user| {
                let full_name = user.full_name();
                UserDto {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    full_name,
                    is_active: user.is_active,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                }
            })
            .collect();

        Ok(PaginatedResponse::new(user_dtos, total, page, page_size))
    }
}
```

Create `src/application/queries/mod.rs`:

```rust
pub mod get_user;
pub mod list_users;

pub use get_user::GetUserQuery;
pub use list_users::ListUsersQuery;
```

Create `src/application/mod.rs`:

```rust
pub mod commands;
pub mod queries;
pub mod dtos;
```

---

### Step 11: Infrastructure Layer - Security (Password Hashing)

Create `src/infrastructure/security/password.rs`:

```rust
use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct PasswordHasher;

impl PasswordHasher {
    pub fn new() -> Self {
        Self
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        hash(password, DEFAULT_COST).context("Failed to hash password")
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        verify(password, hash).context("Failed to verify password")
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}
```

---

### Step 12: Infrastructure Layer - Security (JWT)

Create `src/infrastructure/security/jwt.rs`:

```rust
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub username: String,
    pub exp: i64,     // Expiration time
    pub iat: i64,     // Issued at
}

pub struct JwtManager {
    secret: String,
    expiration_hours: i64,
}

impl JwtManager {
    pub fn new(secret: String, expiration_hours: i64) -> Self {
        Self {
            secret,
            expiration_hours,
        }
    }

    pub fn generate_token(&self, user_id: Uuid, username: &str) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .context("Failed to generate JWT token")
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .context("Failed to verify JWT token")?;

        Ok(token_data.claims)
    }
}
```

Create `src/infrastructure/security/mod.rs`:

```rust
pub mod jwt;
pub mod password;

pub use jwt::{Claims, JwtManager};
pub use password::PasswordHasher;
```

---

### Step 13: Infrastructure Layer - Configuration

Create `src/infrastructure/config.rs`:

```rust
use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            jwt_secret: env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("JWT_EXPIRATION_HOURS must be a valid number")?,
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("SERVER_PORT must be a valid number")?,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
```

---

### Step 14: Infrastructure Layer - Database Repository Implementation

Create `src/infrastructure/database/user_repository_impl.rs`:

```rust
use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: User) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, username, password_hash, first_name, last_name, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.is_active)
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert user into database")?;

        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by id")?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by email")?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by username")?;

        Ok(user)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch users")?;

        Ok(users)
    }

    async fn update(&self, user: User) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $2, first_name = $3, last_name = $4, is_active = $5
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.is_active)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update user")?;

        Ok(user)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to delete user")?;

        Ok(result.rows_affected() > 0)
    }

    async fn count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count users")?;

        Ok(count.0)
    }
}
```

Create `src/infrastructure/database/mod.rs`:

```rust
pub mod user_repository_impl;
pub use user_repository_impl::PostgresUserRepository;
```

Create `src/infrastructure/mod.rs`:

```rust
pub mod config;
pub mod database;
pub mod security;
```

---

### Step 15: Presentation Layer - Authentication Middleware

Create `src/presentation/middleware/auth_middleware.rs`:

```rust
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
```

Create `src/presentation/middleware/mod.rs`:

```rust
pub mod auth_middleware;
pub use auth_middleware::{auth_middleware, AuthUser};
```

---

### Step 16: Presentation Layer - Authentication Handlers

Create `src/presentation/handlers/auth_handlers.rs`:

```rust
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
```

Create `src/presentation/handlers/mod.rs`:

```rust
pub mod auth_handlers;
pub mod user_handlers;
```

---

### Step 17: Presentation Layer - User Handlers

Create `src/presentation/handlers/user_handlers.rs`:

```rust
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
```

---

### Step 18: Presentation Layer - Routes

Create `src/presentation/routes/user_routes.rs`:

```rust
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
```

Create `src/presentation/routes/mod.rs`:

```rust
pub mod user_routes;
pub use user_routes::user_routes;
```

Create `src/presentation/mod.rs`:

```rust
pub mod handlers;
pub mod middleware;
pub mod routes;
```

---

### Step 19: Main Application Entry Point

Create `src/main.rs`:

```rust
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
```

---

### Step 20: Create Integration Tests

Create `tests/integration_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use serde_json::json;

    // Note: For full integration tests, you would:
    // 1. Set up a test database
    // 2. Run migrations
    // 3. Start the server
    // 4. Make HTTP requests using reqwest

    // Example test structure (requires test database setup):

    /*
    use reqwest;

    #[tokio::test]
    async fn test_user_registration_and_login() {
        let client = reqwest::Client::new();
        let base_url = "http://localhost:3000/api";

        // 1. Register a new user
        let create_response = client
            .post(format!("{}/users", base_url))
            .json(&json!({
                "email": "test@example.com",
                "username": "testuser",
                "password": "securepassword123",
                "first_name": "Test",
                "last_name": "User"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(create_response.status(), 201);

        // 2. Login
        let login_response = client
            .post(format!("{}/auth/login", base_url))
            .json(&json!({
                "username": "testuser",
                "password": "securepassword123"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(login_response.status(), 200);
        let auth_data: serde_json::Value = login_response.json().await.unwrap();
        let token = auth_data["token"].as_str().unwrap();

        // 3. Get user list (authenticated)
        let list_response = client
            .get(format!("{}/users", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        assert_eq!(list_response.status(), 200);
    }
    */

    #[test]
    fn test_placeholder() {
        // Placeholder test - implement actual tests as shown above
        assert!(true);
    }
}
```

---

## Testing the API

### Installation and Build

1. **Install dependencies** (already in Cargo.toml):
```bash
cargo check
```

2. **Run database migrations**:
```bash
sqlx migrate run
```

3. **Build the project**:
```bash
cargo build
```

4. **Run the application**:
```bash
cargo run
```

The server will start on `http://127.0.0.1:3000`.

---

### Manual Testing with curl

#### 1. Register a New User

```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "username": "johndoe",
    "password": "SecurePass123",
    "first_name": "John",
    "last_name": "Doe"
  }'
```

**Expected Response** (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "john.doe@example.com",
  "username": "johndoe",
  "first_name": "John",
  "last_name": "Doe",
  "full_name": "John Doe",
  "is_active": true,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

#### 2. Login

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "password": "SecurePass123"
  }'
```

**Expected Response** (200 OK):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "john.doe@example.com",
    "username": "johndoe",
    "first_name": "John",
    "last_name": "Doe",
    "full_name": "John Doe",
    "is_active": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

**Save the token** for subsequent requests:
```bash
export TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

#### 3. Get All Users (Protected)

```bash
curl -X GET http://localhost:3000/api/users \
  -H "Authorization: Bearer $TOKEN"
```

**Expected Response** (200 OK):
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "john.doe@example.com",
      "username": "johndoe",
      "first_name": "John",
      "last_name": "Doe",
      "full_name": "John Doe",
      "is_active": true,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 10,
  "total_pages": 1
}
```

#### 4. Get User by ID (Protected)

```bash
curl -X GET http://localhost:3000/api/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN"
```

#### 5. Update User (Protected)

```bash
curl -X PUT http://localhost:3000/api/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "Jonathan",
    "last_name": "Doe"
  }'
```

#### 6. Delete User (Protected)

```bash
curl -X DELETE http://localhost:3000/api/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer $TOKEN"
```

**Expected Response**: 204 No Content

---

## Running the Application

### Development Mode

```bash
# Run with auto-reload (install cargo-watch first)
cargo install cargo-watch
cargo watch -x run
```

### Production Mode

```bash
# Build optimized binary
cargo build --release

# Run
./target/release/crud-api-project
```

### Using Docker (Optional)

Create `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/crud-api-project /usr/local/bin/
CMD ["crud-api-project"]
```

Build and run:
```bash
docker build -t crud-api .
docker run -p 3000:3000 --env-file .env crud-api
```

---

## Key Learning Points

### 1. Clean Architecture in Rust
- **Domain** layer has no dependencies on other layers
- **Application** layer orchestrates business logic
- **Infrastructure** implements technical details
- **Presentation** handles HTTP communication
- Dependencies point inward (Dependency Inversion Principle)

### 2. CQRS Pattern
- **Commands**: `CreateUserCommand`, `UpdateUserCommand`, `DeleteUserCommand`
- **Queries**: `GetUserQuery`, `ListUsersQuery`
- Clear separation of read and write concerns
- Enables independent optimization

### 3. Dependency Injection in Rust
- **Traits** define abstractions (`UserRepository`)
- **Arc** for shared ownership across async tasks
- **Type system** ensures compile-time correctness
- Manual DI setup in `main.rs` (no magic framework)

### 4. Security
- **Password hashing** with bcrypt
- **JWT tokens** for stateless authentication
- **Middleware** for route protection
- **Validation** with validator crate

### 5. Async/Await
- All I/O operations are async
- **Tokio** runtime for concurrency
- **Axum** for modern async web framework

---

## Next Steps

1. **Add more features**:
   - Email verification
   - Password reset
   - Role-based access control (RBAC)
   - Audit logging

2. **Improve testing**:
   - Unit tests for each layer
   - Integration tests with test database
   - Load testing

3. **Production readiness**:
   - Connection pooling optimization
   - Rate limiting
   - API documentation (OpenAPI/Swagger)
   - Monitoring and metrics
   - Docker Compose setup

4. **Advanced patterns**:
   - Event sourcing
   - Domain events
   - Outbox pattern for reliable messaging

---

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
pg_isready

# Test connection
psql postgresql://crud_api_user:your_secure_password@localhost/crud_api_db
```

### Migration Issues

```bash
# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Compilation Issues

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
```

---

## Conclusion

Congratulations! You've built a production-ready CRUD API in Rust with:
- Clean Architecture for maintainability
- CQRS for scalable read/write separation
- Dependency Injection for testability
- JWT authentication for security
- Async/await for performance

This architecture is suitable for real-world applications and demonstrates modern Rust best practices.

**Happy coding!**
