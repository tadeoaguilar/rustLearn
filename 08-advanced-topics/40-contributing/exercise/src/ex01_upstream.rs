//! Exercise 1: fixing reported bugs in an "upstream" library.
//!
//! This file is a small text-humanizing crate as you'd find it on
//! crates.io: it works for the common cases, and users have filed four
//! issues (see exercises.md). Unlike the other exercises, the code here is
//! *complete* -- and buggy. For each issue:
//!
//! 1. reproduce it: run the issue's example (`cargo run -p m40-contributing -- 1`)
//! 2. write a regression test that fails (the module's tests include one per
//!    issue, `ex1_issue_N_*`; in a real project you'd add it to the crate)
//! 3. fix it with the smallest correct change, keeping every existing
//!    behaviour (`ex1_existing_*` tests) and the public API unchanged
//! 4. write the commit message and PR description (exercises.md)

use std::fmt;
use std::time::Duration;

/// Shorten `text` to at most `max_chars` characters, ending with `…` if
/// anything was cut. Text that fits is returned unchanged.
pub fn truncate(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    format!("{}…", &text[..max_chars - 1])
}

/// `1536` -> `"1.5 KiB"`; below 1024, `"N B"`. One decimal, binary units up to TiB.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        return format!("{bytes} B");
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    InvalidChar(char),
    MissingUnit,
    Overflow,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty duration"),
            ParseError::InvalidChar(c) => write!(f, "invalid character {c:?}"),
            ParseError::MissingUnit => write!(f, "a number without a unit (h, m or s)"),
            ParseError::Overflow => write!(f, "duration too large"),
        }
    }
}

impl std::error::Error for ParseError {}

/// `"1h30m"`, `"90s"`, `"2h 5s"` (spaces ignored) -> a `Duration`.
pub fn parse_duration(text: &str) -> Result<Duration, ParseError> {
    let mut total: u64 = 0;
    let mut number: u64 = 0;
    for c in text.chars() {
        match c {
            ' ' => {}
            '0'..='9' => number = number * 10 + c.to_digit(10).unwrap() as u64,
            'h' => {
                total += number * 3600;
                number = 0;
            }
            'm' => {
                total += number * 60;
                number = 0;
            }
            's' => {
                total += number;
                number = 0;
            }
            other => return Err(ParseError::InvalidChar(other)),
        }
    }
    Ok(Duration::from_secs(total))
}
