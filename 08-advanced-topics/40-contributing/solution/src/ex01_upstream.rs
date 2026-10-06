//! Exercise 1: fixing reported bugs in an "upstream" library.
//!
//! This file is a small text-humanizing crate as you'd find it on
//! crates.io: it works for the common cases, and users have filed four
//! issues (exercises.md). The exercise crate contains the code *with* the
//! bugs. For each issue: reproduce it in a test, fix it with the smallest
//! change that's correct, and keep every existing behaviour and the public
//! API unchanged -- that's what makes a patch easy to merge.
//!
//! The fixes here are marked `// fix #N`.

use std::fmt;
use std::time::Duration;

/// Shorten `text` to at most `max_chars` characters, ending with `…` if
/// anything was cut. Text that fits is returned unchanged.
pub fn truncate(text: &str, max_chars: usize) -> String {
    // fix #1: count and cut by characters, not bytes; and handle max 0
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let kept: String = text.chars().take(max_chars - 1).collect();
    format!("{kept}…")
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
    // fix #2: 1023.95+ rounds to "1024.0" -- promote to the next unit
    if (value * 10.0).round() / 10.0 >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
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
    let mut number: Option<u64> = None;
    let mut saw_any = false;
    for c in text.chars() {
        match c {
            ' ' => {}
            '0'..='9' => {
                let digit = c.to_digit(10).expect("a digit") as u64;
                // fix #4: checked arithmetic instead of overflowing (a panic in debug, garbage in release)
                number = Some(
                    number
                        .unwrap_or(0)
                        .checked_mul(10)
                        .and_then(|n| n.checked_add(digit))
                        .ok_or(ParseError::Overflow)?,
                );
            }
            'h' | 'm' | 's' => {
                let seconds_per = match c {
                    'h' => 3600,
                    'm' => 60,
                    _ => 1,
                };
                // a unit without a number ("h") is as wrong as a number without a unit
                let n = number.take().ok_or(ParseError::InvalidChar(c))?;
                total = n
                    .checked_mul(seconds_per)
                    .and_then(|s| total.checked_add(s))
                    .ok_or(ParseError::Overflow)?;
                saw_any = true;
            }
            other => return Err(ParseError::InvalidChar(other)),
        }
    }
    // fix #3: a trailing number was silently dropped, and "" was 0 seconds
    if number.is_some() {
        return Err(ParseError::MissingUnit);
    }
    if !saw_any {
        return Err(ParseError::Empty);
    }
    Ok(Duration::from_secs(total))
}
