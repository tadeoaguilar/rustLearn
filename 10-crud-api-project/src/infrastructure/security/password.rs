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
