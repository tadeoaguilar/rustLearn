//! Exercise 6: Pattern Matching.

/// Task 1: `|` for alternatives, `..=` for inclusive ranges.
pub fn describe_number(n: i32) -> &'static str {
    todo!("Exercise 6")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Task 2: destructuring with literals inside the pattern.
pub fn locate(point: Point) -> String {
    todo!("Exercise 6")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Temperature {
    Celsius(i32),
    Fahrenheit(i32),
}

/// Task 3: guards. Note the `_` arm is required: the compiler does not reason
/// about guard conditions, so without it the match is "non-exhaustive" even if
/// you covered every number.
pub fn describe_temperature(temp: Temperature) -> &'static str {
    todo!("Exercise 6")
}

/// Task 4: `if let` for "I only care about one pattern".
pub fn is_three(value: Option<i32>) -> bool {
    todo!("Exercise 6")
}

/// Task 4: `while let` loops as long as the pattern matches -- here, until
/// `pop` returns None.
pub fn drain_stack(mut stack: Vec<i32>) -> Vec<i32> {
    todo!("Exercise 6")
}

/// Bonus pattern: `let else` (Rust 1.65+) -- bind the pattern, or run the
/// `else` block, which must leave the function (return, break, panic...).
/// Useful when the early exit isn't just "return None" -- for that, `?` is
/// shorter, as in `parse_pair` below.
pub fn describe_pair(s: &str) -> String {
    todo!("Exercise 6")
}

/// `?` on Option: any None returns None from the whole function.
pub fn parse_pair(s: &str) -> Option<(i32, i32)> {
    todo!("Exercise 6")
}

pub fn run() {
    todo!("Exercise 6")
}
