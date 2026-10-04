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

// BUG FIXED: the original table had no (900, "CM") and (90, "XC") entries,
// so 90 came out as "LXXXX" and 900 as "DCCCC".
const TABLE: [(u32, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

/// Writes `n` as a canonical Roman numeral.
///
/// ```
/// use m09_testing_solution::roman::to_roman;
/// assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
/// assert_eq!(to_roman(90).unwrap(), "XC");
/// assert!(to_roman(0).is_err());
/// ```
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

    #[test]
    fn known_values() {
        for (n, s) in [
            (1, "I"),
            (4, "IV"),
            (9, "IX"),
            (14, "XIV"),
            (40, "XL"),
            (90, "XC"),
            (400, "CD"),
            (900, "CM"),
            (1994, "MCMXCIV"),
            (2024, "MMXXIV"),
            (3999, "MMMCMXCIX"),
        ] {
            assert_eq!(to_roman(n).as_deref(), Ok(s), "to_roman({n})");
            assert_eq!(from_roman(s), Ok(n), "from_roman({s})");
        }
    }

    #[test]
    fn out_of_range() {
        assert_eq!(to_roman(0), Err(RomanError::OutOfRange(0)));
        assert_eq!(to_roman(4000), Err(RomanError::OutOfRange(4000)));
    }

    #[test]
    fn from_roman_errors_and_leniency() {
        assert_eq!(from_roman(""), Err(RomanError::Empty));
        assert_eq!(from_roman("XIZ"), Err(RomanError::InvalidChar('Z')));
        assert_eq!(from_roman("mcm"), Ok(1900), "lower case accepted");
        assert_eq!(from_roman("IIII"), Ok(4), "non-canonical, but accepted");
    }
}
