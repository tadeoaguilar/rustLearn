//! Discounts. Compiled only with feature `discounts` (on by default).

use crate::rounding::round_half_up;
use m08_money_solution::Money;

/// A discount applied to a cart's subtotal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive] // we may add kinds later without a breaking change
pub enum Discount {
    /// A percentage off, 0..=100.
    Percent(u8),
    /// A fixed amount off; the total never goes below zero.
    Fixed(Money),
    /// A percentage off, only when buying at least `min_items` copies.
    Bulk {
        /// Minimum number of copies in the cart.
        min_items: u32,
        /// Percentage off, 0..=100.
        percent: u8,
    },
}

impl Discount {
    /// The discounted total.
    ///
    /// ```
    /// use m08_modules_crates_solution::{Discount, Money};
    /// let ten_percent = Discount::Percent(10);
    /// assert_eq!(ten_percent.apply(Money::new(19, 99), 1), Money::new(17, 99));
    /// ```
    pub fn apply(&self, subtotal: Money, item_count: u32) -> Money {
        match *self {
            Discount::Percent(p) => percent_off(subtotal, p),
            Discount::Fixed(amount) => {
                let after = subtotal - amount;
                if after.is_negative() {
                    Money::ZERO
                } else {
                    after
                }
            }
            Discount::Bulk { min_items, percent } if item_count >= min_items => {
                percent_off(subtotal, percent)
            }
            Discount::Bulk { .. } => subtotal,
        }
    }
}

fn percent_off(subtotal: Money, percent: u8) -> Money {
    let percent = i64::from(percent.min(100));
    let off = round_half_up(subtotal.cents() * percent, 100);
    Money::from_cents(subtotal.cents() - off)
}
