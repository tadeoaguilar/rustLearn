//! Bonus Challenge: Result Combinators.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub username: String,
    pub email: String,
    pub age: u32,
}

pub fn validate_username(username: &str) -> Result<String, String> {
    match username.chars().count() {
        0..=2 => Err("Username too short".to_string()),
        3..=20 => Ok(username.to_string()),
        _ => Err("Username too long".to_string()),
    }
}

pub fn validate_email(email: &str) -> Result<String, String> {
    // Still not real email validation -- but "a@" or "@b" shouldn't pass.
    match email.split_once('@') {
        Some((local, domain)) if !local.is_empty() && domain.contains('.') => Ok(email.to_string()),
        _ => Err("Invalid email format".to_string()),
    }
}

pub fn validate_age(age: u32) -> Result<u32, String> {
    match age {
        0..=12 => Err("Must be at least 13 years old".to_string()),
        13..=120 => Ok(age),
        _ => Err("Age is unrealistic".to_string()),
    }
}

/// Collects *every* validation error, so the user can fix everything at once.
///
/// The exercise's version checks each result, then `unwrap()`s all three.
/// That's correct but fragile: the unwraps are only safe because of the
/// checks above them. Matching on the tuple of results lets the compiler
/// prove it instead -- there's no unwrap left to get wrong.
pub fn register_user(username: &str, email: &str, age: u32) -> Result<User, Vec<String>> {
    match (
        validate_username(username),
        validate_email(email),
        validate_age(age),
    ) {
        (Ok(username), Ok(email), Ok(age)) => Ok(User {
            username,
            email,
            age,
        }),
        (u, e, a) => Err([u.err(), e.err(), a.err()].into_iter().flatten().collect()),
    }
}

/// Fail-fast alternative: `?` stops at the first error. Use this when later
/// checks depend on earlier ones, or one message is enough.
pub fn register_user_fail_fast(username: &str, email: &str, age: u32) -> Result<User, String> {
    Ok(User {
        username: validate_username(username)?,
        email: validate_email(email)?,
        age: validate_age(age)?,
    })
}

/// A tour of the combinators, each on one line.
pub fn combinator_tour(input: &str) -> Result<u32, String> {
    input
        .trim()
        .parse::<u32>() // Result<u32, ParseIntError>
        .map_err(|e| format!("'{input}' is not a number: {e}")) // change the error type
        .and_then(validate_age) // chain another fallible step
        .map(|age| age + 1) // transform the success value
        .or_else(|e| {
            if input.trim() == "unknown" {
                Ok(0)
            } else {
                Err(e)
            }
        }) // recover from some errors
}

pub fn run() {
    match register_user("ab", "invalid", 10) {
        Ok(user) => println!("User registered: {user:?}"),
        Err(errors) => {
            println!("Registration failed:");
            for error in errors {
                println!("  - {error}");
            }
        }
    }
    println!("{:?}", register_user("ferris", "ferris@rust-lang.org", 9));
    println!("{:?}", register_user("ferris", "ferris@rust-lang.org", 30));
    println!(
        "fail fast: {:?}",
        register_user_fail_fast("ab", "invalid", 10)
    );
    for input in ["30", "abc", "5", "unknown"] {
        println!("combinator_tour({input:?}) = {:?}", combinator_tour(input));
    }
}
