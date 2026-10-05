//! Exercise 1: password hashing with Argon2id.
//!
//! Never store passwords, encrypted or otherwise -- store a *slow, salted
//! hash*. Argon2id is the current recommendation (OWASP): deliberately
//! expensive in time and memory, so an attacker with a stolen database can
//! try thousands, not billions, of guesses per second. A random salt per
//! password means identical passwords get different hashes.
//!
//! The result is a PHC string that carries everything needed to verify it:
//! `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

/// Hashes with a fresh random salt (argon2 0.6 generates it for you).
pub fn hash_password(password: &str) -> String {
    todo!("Exercise 1")
}

/// `false` for a wrong password *and* for a malformed hash -- callers only
/// need to know "yes" or "no". The comparison inside is constant-time.
pub fn verify_password(password: &str, phc: &str) -> bool {
    todo!("Exercise 1")
}

/// A tiny sample; real systems check against breach corpora (e.g. the
/// "Have I Been Pwned" k-anonymity API).
const COMMON: [&str; 6] = [
    "password1234",
    "123456789012",
    "qwertyuiopas",
    "iloveyou1234",
    "letmein12345",
    "passwordpassword",
];

/// NIST SP 800-63B: length matters, composition rules ("one symbol...")
/// don't. At least 12 characters, at most 128, not a known common password.
pub fn check_policy(password: &str) -> Result<(), String> {
    todo!("Exercise 1")
}
