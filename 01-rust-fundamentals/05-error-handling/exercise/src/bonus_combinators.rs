//! Bonus Challenge: Result Combinators.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub email: String,
    pub age: u32,
}

pub fn validate_username(username: &str) -> Result<String, String> {
    todo!("Bonus")
}

pub fn validate_email(email: &str) -> Result<String, String> {
    todo!("Bonus")
}

pub fn validate_age(age: u32) -> Result<u32, String> {
    todo!("Bonus")
}

/// Collects *every* validation error, so the user can fix everything at once.
///
/// The exercise's version checks each result, then `unwrap()`s all three.
/// That's correct but fragile: the unwraps are only safe because of the
/// checks above them. Matching on the tuple of results lets the compiler
/// prove it instead -- there's no unwrap left to get wrong.
pub fn register_user(username: &str, email: &str, age: u32) -> Result<User, Vec<String>> {
    todo!("Bonus")
}

/// Fail-fast alternative: `?` stops at the first error. Use this when later
/// checks depend on earlier ones, or one message is enough.
pub fn register_user_fail_fast(username: &str, email: &str, age: u32) -> Result<User, String> {
    todo!("Bonus")
}

/// A tour of the combinators, each on one line.
pub fn combinator_tour(input: &str) -> Result<u32, String> {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
