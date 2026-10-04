//! Exercise 2: Cargo Basics -- the `my_math` library.
//!
//! In a real project `my_math` would be its own crate created with
//! `cargo new --lib my_math`. Here it is a module of this library so the whole
//! module builds with one command; `src/main.rs` plays the role of "a binary
//! that uses this library" by calling `m01_getting_started_solution::...`.
//!
//! The written answers to this exercise's questions are in `../ANSWERS.md`.

/// Adds two numbers.
///
/// Doc comments are compiled *and run* as tests by `cargo test`:
///
/// ```
/// use m01_getting_started_solution::ex02_cargo_basics::add;
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    // Unit tests live next to the code they test, in a `#[cfg(test)]` module
    // that is compiled only for `cargo test`.
    use super::*;

    #[test]
    fn adds_positive_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn adds_negative_numbers() {
        assert_eq!(add(-2, -3), -5);
    }
}
