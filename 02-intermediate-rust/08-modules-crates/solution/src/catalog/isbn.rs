//! ISBN-13 numbers.

use std::fmt;

/// A validated ISBN-13, stored as its 13 digits without hyphens.
///
/// The field is private: the only way to get an `Isbn` is [`Isbn::parse`], so
/// every `Isbn` in the program is known to be valid. This is the "parse, don't
/// validate" pattern.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct Isbn(String);

/// Why a string isn't a valid ISBN-13.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsbnError {
    /// Not 13 digits (after removing hyphens). Holds the digit count found.
    WrongLength(usize),
    /// Contains something other than digits and hyphens.
    InvalidCharacter(char),
    /// The 13th digit doesn't match the checksum.
    BadChecksum,
}

impl fmt::Display for IsbnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsbnError::WrongLength(n) => write!(f, "an ISBN-13 has 13 digits, found {n}"),
            IsbnError::InvalidCharacter(c) => write!(f, "invalid character {c:?} in ISBN"),
            IsbnError::BadChecksum => write!(f, "ISBN checksum does not match"),
        }
    }
}

impl std::error::Error for IsbnError {}

impl Isbn {
    /// Parses an ISBN-13, with or without hyphens.
    ///
    /// ```
    /// use m08_modules_crates_solution::catalog::{Isbn, IsbnError};
    ///
    /// let isbn = Isbn::parse("978-0-306-40615-7")?;
    /// assert_eq!(isbn.to_string(), "9780306406157");
    /// assert_eq!(Isbn::parse("978-0-306-40615-8"), Err(IsbnError::BadChecksum));
    /// # Ok::<(), IsbnError>(())
    /// ```
    ///
    /// There is no other way to construct one -- the field is private:
    ///
    /// ```compile_fail
    /// use m08_modules_crates_solution::catalog::Isbn;
    /// let isbn = Isbn("not validated".to_string()); // error[E0603]: tuple struct constructor `Isbn` is private
    /// ```
    pub fn parse(s: &str) -> Result<Isbn, IsbnError> {
        let mut digits = String::with_capacity(13);
        for c in s.trim().chars() {
            match c {
                '0'..='9' => digits.push(c),
                '-' | ' ' => {}
                other => return Err(IsbnError::InvalidCharacter(other)),
            }
        }
        if digits.len() != 13 {
            return Err(IsbnError::WrongLength(digits.len()));
        }
        // Weights alternate 1, 3, 1, 3, ...; the weighted sum of all 13 digits
        // must be a multiple of 10.
        let sum: u32 = digits
            .bytes()
            .enumerate()
            .map(|(i, b)| u32::from(b - b'0') * if i % 2 == 0 { 1 } else { 3 })
            .sum();
        if !sum.is_multiple_of(10) {
            return Err(IsbnError::BadChecksum);
        }
        Ok(Isbn(digits))
    }

    /// The 13 digits.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Isbn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
