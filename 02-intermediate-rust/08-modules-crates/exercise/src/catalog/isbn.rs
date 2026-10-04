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
        todo!("Exercise 2")
    }
}

impl std::error::Error for IsbnError {}

impl Isbn {
    /// Parses an ISBN-13, with or without hyphens.
    ///
    /// ```
    /// use m08_modules_crates::catalog::{Isbn, IsbnError};
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
    /// use m08_modules_crates::catalog::Isbn;
    /// let isbn = Isbn("not validated".to_string()); // error[E0603]: tuple struct constructor `Isbn` is private
    /// ```
    pub fn parse(s: &str) -> Result<Isbn, IsbnError> {
        todo!("Exercise 2")
    }

    /// The 13 digits.
    pub fn as_str(&self) -> &str {
        todo!("Exercise 2")
    }
}

impl fmt::Display for Isbn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 2")
    }
}
