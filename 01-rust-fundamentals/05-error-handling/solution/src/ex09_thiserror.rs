//! Exercise 9: Using the thiserror Crate.
//!
//! `#[derive(Error)]` writes the Display impl from the `#[error(...)]`
//! attributes, the `Error` impl, and -- for `#[from]` fields -- the `From`
//! impls and `source()`. Compare with the ~40 hand-written lines of MyError in
//! ex05. thiserror is for *libraries*; for applications see `anyhow` below.

use std::io;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseIntError),

    #[error("Invalid value: {value} (must be between {min} and {max})")]
    OutOfRange { value: i32, min: i32, max: i32 },

    #[error("Custom error: {0}")]
    Custom(String),
}

pub fn validate_percentage(value: i32) -> Result<i32, DataError> {
    if !(0..=100).contains(&value) {
        Err(DataError::OutOfRange {
            value,
            min: 0,
            max: 100,
        })
    } else {
        Ok(value)
    }
}

/// `?` uses the generated `From<ParseIntError>`.
pub fn parse_percentage(s: &str) -> Result<i32, DataError> {
    let value: i32 = s.trim().parse()?;
    validate_percentage(value)
}

/// anyhow, for applications: one error type for everything, plus `context`
/// to say *what you were doing* when it failed. Printing with `{:#}` shows
/// the whole chain: "reading percentage from x.txt: No such file or directory".
pub fn read_percentage_file(path: &std::path::Path) -> anyhow::Result<i32> {
    use anyhow::Context;
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading percentage from {}", path.display()))?;
    let value = parse_percentage(&text).context("parsing percentage")?;
    Ok(value)
}

pub fn run() {
    for input in ["50", "150", "abc"] {
        match parse_percentage(input) {
            Ok(v) => println!("{input:>4}: Valid: {v}"),
            Err(e) => println!("{input:>4}: Error: {e}"),
        }
    }
    println!("{}", DataError::Custom("something odd".into()));
    if let Err(e) = read_percentage_file(std::path::Path::new("/no/such/percentage.txt")) {
        println!("anyhow {{}}:   {e}");
        println!("anyhow {{:#}}: {e:#}");
    }
}
