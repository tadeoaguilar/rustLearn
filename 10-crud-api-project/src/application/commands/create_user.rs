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
