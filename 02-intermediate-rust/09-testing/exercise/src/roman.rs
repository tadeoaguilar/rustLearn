//! Roman numerals, 1 to 3999.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange(u32),
    Empty,
    InvalidChar(char),
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "{n} can't be written in Roman numerals (1..=3999)")
            }
            RomanError::Empty => write!(f, "empty numeral"),
            RomanError::InvalidChar(c) => write!(f, "{c:?} is not a Roman numeral"),
        }
    }
}

impl std::error::Error for RomanError {}

const TABLE: [(u32, &str); 11] = [
    (1000, "M"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

/// Writes `n` as a canonical Roman numeral.
pub fn to_roman(mut n: u32) -> Result<String, RomanError> {
    if !(1..=3999).contains(&n) {
        return Err(RomanError::OutOfRange(n));
    }
    let mut out = String::new();
    for &(value, symbol) in &TABLE {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }
    Ok(out)
}

fn value_of(c: char) -> Result<u32, RomanError> {
    Ok(match c.to_ascii_uppercase() {
        'I' => 1,
        'V' => 5,
        'X' => 10,
        'L' => 50,
        'C' => 100,
        'D' => 500,
        'M' => 1000,
        other => return Err(RomanError::InvalidChar(other)),
    })
}

/// Parses a Roman numeral. Lenient: it also accepts non-canonical forms like
/// `IIII` -- which is exactly why a roundtrip test alone can't catch a
/// `to_roman` that produces them.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    let values = s
        .trim()
        .chars()
        .map(value_of)
        .collect::<Result<Vec<u32>, _>>()?;
    if values.is_empty() {
        return Err(RomanError::Empty);
    }
    let mut total = 0;
    for (i, &v) in values.iter().enumerate() {
        // A smaller value before a larger one is subtracted: the I in IV.
        match values.get(i + 1) {
            Some(&next) if v < next => total -= v as i64,
            _ => total += v as i64,
        }
    }
    u32::try_from(total).map_err(|_| RomanError::InvalidChar('?'))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 6: your tests here.
}
