//! Tests for 08-modules-crates.
//!
//! `sut` is the bookstore library and `money` the money crate -- either the
//! reference solution or your exercise crates, chosen by the `mine` feature.

#[cfg(not(feature = "mine"))]
pub use {m08_modules_crates_solution as sut, m08_money_solution as money};

#[cfg(feature = "mine")]
pub use {m08_modules_crates as sut, m08_money as money};

#[cfg(test)]
mod exercises;
