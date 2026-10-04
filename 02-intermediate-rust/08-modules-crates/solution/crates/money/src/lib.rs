//! Money as an integer number of cents.
//!
//! Never store money in `f64`: `0.1 + 0.2 != 0.3`, and those errors add up.
//! An `i64` of cents is exact for any amount you'll realistically handle.
//!
//! ```
//! use m08_money_solution::Money;
//!
//! let price: Money = "$12.34".parse()?;
//! assert_eq!(price * 3, Money::from_cents(3702));
//! assert_eq!((price * 3).to_string(), "$37.02");
//! # Ok::<(), m08_money_solution::ParseMoneyError>(())
//! ```

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
        Money { cents }
    }

    /// From dollars and cents: `new(12, 34)` is $12.34.
    pub const fn new(dollars: i64, cents: i64) -> Self {
        Money {
            cents: dollars * 100 + cents,
        }
    }

    /// The amount in cents.
    pub const fn cents(self) -> i64 {
        self.cents
    }

    /// True if the amount is below zero.
    pub const fn is_negative(self) -> bool {
        self.cents < 0
    }
}

/// `$12.34`, `-$0.05`.
///
/// ```
/// use m08_money_solution::Money;
/// assert_eq!(Money::from_cents(1234).to_string(), "$12.34");
/// assert_eq!(Money::from_cents(-5).to_string(), "-$0.05");
/// ```
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.cents < 0 { "-" } else { "" };
        let abs = self.cents.unsigned_abs();
        write!(f, "{sign}${}.{:02}", abs / 100, abs % 100)
    }
}

/// Why a string couldn't be parsed as [`Money`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMoneyError(String);

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid money amount: {}", self.0)
    }
}

impl std::error::Error for ParseMoneyError {}

/// Parses `12`, `12.3`, `12.34`, `$12.34`, `-$1.50`.
impl FromStr for Money {
    type Err = ParseMoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseMoneyError(s.to_string());
        let t = s.trim();
        let (negative, t) = match t.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, t),
        };
        let t = t.strip_prefix('$').unwrap_or(t);
        let (dollars, cents) = match t.split_once('.') {
            Some((d, c)) => (d, c),
            None => (t, "0"),
        };
        if dollars.is_empty() || cents.is_empty() || cents.len() > 2 {
            return Err(err());
        }
        let dollars: i64 = dollars.parse().map_err(|_| err())?;
        // "12.3" means 30 cents, not 3.
        let cents: i64 = format!("{cents:0<2}").parse().map_err(|_| err())?;
        let total = dollars
            .checked_mul(100)
            .and_then(|d| d.checked_add(cents))
            .ok_or_else(err)?;
        Ok(Money::from_cents(if negative { -total } else { total }))
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money::from_cents(self.cents + rhs.cents)
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money::from_cents(self.cents - rhs.cents)
    }
}

impl Mul<u32> for Money {
    type Output = Money;
    fn mul(self, quantity: u32) -> Money {
        Money::from_cents(self.cents * i64::from(quantity))
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, Add::add)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        iter.copied().sum()
    }
}
