//! Money as an integer number of cents.
//!
//! Never store money in `f64`: `0.1 + 0.2 != 0.3`, and those errors add up.
//! An `i64` of cents is exact for any amount you'll realistically handle.
//!
//! ```
//! use m08_money::Money;
//!
//! let price: Money = "$12.34".parse()?;
//! assert_eq!(price * 3, Money::from_cents(3702));
//! assert_eq!((price * 3).to_string(), "$37.02");
//! # Ok::<(), m08_money::ParseMoneyError>(())
//! ```

// YOUR WORKSPACE for Exercise 5 -- see ../../../exercises.md.
// Remove this once you have started: it silences "unused" warnings.
#![allow(unused)]
#![warn(missing_docs)]

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

/// An amount of money in cents. Negative amounts are allowed (refunds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Money {
    cents: i64,
}

impl Money {
    /// Zero dollars.
    pub const ZERO: Money = Money { cents: 0 };

    /// From a number of cents: `from_cents(1234)` is $12.34.
    pub const fn from_cents(cents: i64) -> Self {
        todo!()
    }

    /// From dollars and cents: `new(12, 34)` is $12.34.
    pub const fn new(dollars: i64, cents: i64) -> Self {
        todo!()
    }

    /// The amount in cents.
    pub const fn cents(self) -> i64 {
        todo!()
    }

    /// True if the amount is below zero.
    pub const fn is_negative(self) -> bool {
        todo!()
    }
}

/// `$12.34`, `-$0.05`.
///
/// ```
/// use m08_money::Money;
/// assert_eq!(Money::from_cents(1234).to_string(), "$12.34");
/// assert_eq!(Money::from_cents(-5).to_string(), "-$0.05");
/// ```
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 5")
    }
}

/// Why a string couldn't be parsed as [`Money`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMoneyError(String);

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 5")
    }
}

impl std::error::Error for ParseMoneyError {}

/// Parses `12`, `12.3`, `12.34`, `$12.34`, `-$1.50`.
impl FromStr for Money {
    type Err = ParseMoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!("Exercise 5")
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        todo!("Exercise 5")
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        todo!("Exercise 5")
    }
}

impl Mul<u32> for Money {
    type Output = Money;
    fn mul(self, quantity: u32) -> Money {
        todo!("Exercise 5")
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        todo!("Exercise 5")
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        todo!("Exercise 5")
    }
}
