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
