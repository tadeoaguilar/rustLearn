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
