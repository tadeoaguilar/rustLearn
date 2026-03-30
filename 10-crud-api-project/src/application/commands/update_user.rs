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
