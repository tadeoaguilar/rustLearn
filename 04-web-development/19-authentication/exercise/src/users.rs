//! Exercise 1: registration and login.

use crate::password::{check_policy, hash_password, verify_password};
use crate::rbac::Role;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: Option<String>, // None for accounts that only log in via OAuth
    pub role: Role,
    pub totp_secret: Option<Vec<u8>>, // bonus: set once MFA is enabled
    pub oauth_subject: Option<String>, // the provider's id for this user
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UserError {
    #[error("invalid username: {0}")]
    InvalidUsername(String),
    #[error("weak password: {0}")]
    WeakPassword(String),
    #[error("username already taken")]
    Taken,
    /// One error for "no such user" AND "wrong password". Distinct errors
    /// let an attacker discover which usernames exist ("user enumeration").
    #[error("invalid username or password")]
    InvalidCredentials,
}

#[derive(Debug, Clone, Default)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, User>>>,
}

/// Hashed once: verifying against it costs the same as a real check.
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| hash_password("dummy password for timing"));

impl UserStore {
    pub fn new() -> Self {
        todo!("Exercise 1")
    }

    /// The first user to register becomes an admin; everyone else is a user.
    pub fn register(&self, username: &str, password: &str) -> Result<User, UserError> {
        todo!("Exercise 1")
    }

    /// When the user doesn't exist we still run a full Argon2 verification
    /// against a dummy hash. Otherwise "unknown user" answers in 1 ms and
    /// "wrong password" in 50 ms, and the timing leaks which usernames exist.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<User, UserError> {
        todo!("Exercise 1")
    }

    pub fn find(&self, username: &str) -> Option<User> {
        todo!("Exercise 1")
    }

    pub fn find_by_id(&self, id: u64) -> Option<User> {
        todo!("Exercise 1")
    }

    pub fn all(&self) -> Vec<User> {
        todo!("Exercise 1")
    }

    pub fn set_role(&self, id: u64, role: Role) -> Option<User> {
        todo!("Exercise 1")
    }

    pub fn set_totp_secret(&self, id: u64, secret: Vec<u8>) {
        todo!("Exercise 1")
    }

    /// Exercise 6: find the account linked to an OAuth identity, or create one.
    pub fn find_or_create_oauth(&self, subject: &str, preferred_username: &str) -> User {
        todo!("Exercise 1")
    }
}
