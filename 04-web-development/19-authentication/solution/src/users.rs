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
        Self::default()
    }

    /// The first user to register becomes an admin; everyone else is a user.
    pub fn register(&self, username: &str, password: &str) -> Result<User, UserError> {
        let username = username.trim().to_lowercase();
        let valid = (3..=32).contains(&username.len())
            && username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid {
            return Err(UserError::InvalidUsername(
                "3-32 letters, digits or _".into(),
            ));
        }
        check_policy(password).map_err(UserError::WeakPassword)?;
        let hash = hash_password(password); // slow: done *before* taking the lock
        let mut users = self.users.write().unwrap();
        if users.contains_key(&username) {
            return Err(UserError::Taken);
        }
        let role = if users.is_empty() {
            Role::Admin
        } else {
            Role::User
        };
        let user = User {
            id: users.len() as u64 + 1,
            username: username.clone(),
            password_hash: Some(hash),
            role,
            totp_secret: None,
            oauth_subject: None,
        };
        users.insert(username, user.clone());
        Ok(user)
    }

    /// When the user doesn't exist we still run a full Argon2 verification
    /// against a dummy hash. Otherwise "unknown user" answers in 1 ms and
    /// "wrong password" in 50 ms, and the timing leaks which usernames exist.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<User, UserError> {
        let user = self.find(&username.trim().to_lowercase());
        let hash = user
            .as_ref()
            .and_then(|u| u.password_hash.clone())
            .unwrap_or_else(|| DUMMY_HASH.clone());
        let ok = verify_password(password, &hash);
        match user {
            Some(u) if ok && u.password_hash.is_some() => Ok(u),
            _ => Err(UserError::InvalidCredentials),
        }
    }

    pub fn find(&self, username: &str) -> Option<User> {
        self.users.read().unwrap().get(username).cloned()
    }

    pub fn find_by_id(&self, id: u64) -> Option<User> {
        self.users
            .read()
            .unwrap()
            .values()
            .find(|u| u.id == id)
            .cloned()
    }

    pub fn all(&self) -> Vec<User> {
        let mut v: Vec<User> = self.users.read().unwrap().values().cloned().collect();
        v.sort_by_key(|u| u.id);
        v
    }

    pub fn set_role(&self, id: u64, role: Role) -> Option<User> {
        let mut users = self.users.write().unwrap();
        let user = users.values_mut().find(|u| u.id == id)?;
        user.role = role;
        Some(user.clone())
    }

    pub fn set_totp_secret(&self, id: u64, secret: Vec<u8>) {
        if let Some(u) = self
            .users
            .write()
            .unwrap()
            .values_mut()
            .find(|u| u.id == id)
        {
            u.totp_secret = Some(secret);
        }
    }

    /// Exercise 6: find the account linked to an OAuth identity, or create one.
    pub fn find_or_create_oauth(&self, subject: &str, preferred_username: &str) -> User {
        let mut users = self.users.write().unwrap();
        if let Some(u) = users
            .values()
            .find(|u| u.oauth_subject.as_deref() == Some(subject))
        {
            return u.clone();
        }
        let mut username = format!("{}_oauth", preferred_username.to_lowercase());
        while users.contains_key(&username) {
            username.push('_');
        }
        let user = User {
            id: users.len() as u64 + 1,
            username: username.clone(),
            password_hash: None,
            role: Role::User,
            totp_secret: None,
            oauth_subject: Some(subject.to_string()),
        };
        users.insert(username, user.clone());
        user
    }
}
