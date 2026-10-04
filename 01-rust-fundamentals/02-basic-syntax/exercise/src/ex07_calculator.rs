//! Exercise 7: Calculator Program.

use std::io::{self, BufRead, Write};

/// Parses "5 + 3" into (5.0, '+', 3.0).
///
/// `split_whitespace` handles any amount of spacing. Every step that can fail
/// returns `None` through `?` -- yes, `?` works on `Option` as well as `Result`.
pub fn parse_operation(input: &str) -> Option<(f64, char, f64)> {
    todo!("Exercise 7")
}

/// `None` for division by zero (and for an unknown operator).
pub fn calculate(a: f64, op: char, b: f64) -> Option<f64> {
    todo!("Exercise 7")
}

/// The interactive loop. Reads lines until "quit" or end of input.
pub fn run(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    todo!("Exercise 7")
}
