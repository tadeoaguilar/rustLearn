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
        write!(f, "Validation error in {}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_age(age: i32) -> Result<i32, ValidationError> {
    let fail = |message: &str| ValidationError {
        field: "age".to_string(),
        message: message.to_string(),
    };
    if age < 0 {
        Err(fail("Age cannot be negative"))
    } else if age > 150 {
        Err(fail("Age is unrealistically high"))
    } else {
        Ok(age)
    }
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
        match self {
            UserError::InvalidAge(age) => write!(f, "Invalid age: {age}"),
            UserError::InvalidEmail(email) => write!(f, "Invalid email: {email}"),
            UserError::UsernameTooShort(len) => write!(f, "Username too short: {len} characters"),
        }
    }
}

impl std::error::Error for UserError {}

/// Stops at the first problem. (The bonus collects all of them instead.)
pub fn validate_user(age: i32, email: &str, username: &str) -> Result<(), UserError> {
    if !(0..=150).contains(&age) {
        return Err(UserError::InvalidAge(age));
    }
    if !email.contains('@') {
        return Err(UserError::InvalidEmail(email.to_string()));
    }
    // chars().count(), not len(): len() counts bytes, so "日本" would be 6.
    let len = username.chars().count();
    if len < 3 {
        return Err(UserError::UsernameTooShort(len));
    }
    Ok(())
}

pub fn run() {
    for age in [30, -5, 200] {
        match validate_age(age) {
            Ok(age) => println!("Valid age: {age}"),
            Err(e) => println!("Error: {e}"),
        }
    }
    for (age, email, name) in [
        (30, "a@b.c", "alice"),
        (30, "nope", "alice"),
        (30, "a@b.c", "al"),
    ] {
        match validate_user(age, email, name) {
            Ok(()) => println!("{name}: ok"),
            Err(e) => println!("{name}: {e}"),
        }
    }
}
