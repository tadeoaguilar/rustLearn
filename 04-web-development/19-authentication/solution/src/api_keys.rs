//! Exercise 5: API keys for machine clients.
//!
//! Like passwords, keys are stored *hashed*: a database leak shouldn't leak
//! working credentials. Unlike passwords they're long and random, so a fast
//! hash (SHA-256) is enough -- there's nothing to brute-force -- and it lets
//! us look keys up by hash directly.
//!
//! The plaintext key is shown exactly once, at creation. A recognisable
//! prefix (`rlk_`) helps secret scanners (GitHub, gitleaks) spot leaked keys.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyRecord {
    pub user_id: u64,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyStore {
    by_hash: Arc<Mutex<HashMap<String, ApiKeyRecord>>>,
}

pub fn hash_key(key: &str) -> String {
    Sha256::digest(key.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl ApiKeyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the plaintext key. Only its hash is kept.
    pub fn create(&self, user_id: u64, label: &str) -> String {
        let key = format!("rlk_{}", crate::random_hex(24));
        self.by_hash.lock().unwrap().insert(
            hash_key(&key),
            ApiKeyRecord {
                user_id,
                label: label.to_string(),
            },
        );
        key
    }

    pub fn verify(&self, key: &str) -> Option<ApiKeyRecord> {
        if !key.starts_with("rlk_") {
            return None;
        }
        self.by_hash.lock().unwrap().get(&hash_key(key)).cloned()
    }

    pub fn revoke(&self, key: &str) -> bool {
        self.by_hash
            .lock()
            .unwrap()
            .remove(&hash_key(key))
            .is_some()
    }

    /// What a database dump would contain: hashes, not keys.
    pub fn stored_hashes(&self) -> Vec<String> {
        self.by_hash.lock().unwrap().keys().cloned().collect()
    }
}
