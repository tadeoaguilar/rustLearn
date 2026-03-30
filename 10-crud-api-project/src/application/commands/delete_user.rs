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
