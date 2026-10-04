//! Exercise 4: Custom Error Types.
//!
//! A proper error type implements three traits:
//!   * `Debug`              -- for developers ({:?}, unwrap messages)
//!   * `Display`            -- for users ({})
//!   * `std::error::Error`  -- so it fits into Box<dyn Error>, anyhow, `source()` chains

use std::fmt;

/// Task 1: a struct error -- one kind of failure, with context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 4")
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_age(age: i32) -> Result<i32, ValidationError> {
    todo!("Exercise 4")
}

/// Task 2: an enum error -- several kinds of failure, each with its own data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    InvalidAge(i32),
    InvalidEmail(String),
    UsernameTooShort(usize),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 4")
    }
}

impl std::error::Error for UserError {}

/// Stops at the first problem. (The bonus collects all of them instead.)
pub fn validate_user(age: i32, email: &str, username: &str) -> Result<(), UserError> {
    todo!("Exercise 4")
}

pub fn run() {
    todo!("Exercise 4")
}
