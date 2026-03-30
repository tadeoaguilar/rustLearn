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
