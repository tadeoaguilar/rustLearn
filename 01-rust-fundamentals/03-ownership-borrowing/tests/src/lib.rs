//! Tests for 03-ownership-borrowing.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m03_ownership_borrowing_solution as sut;

#[cfg(feature = "mine")]
pub use m03_ownership_borrowing as sut;

#[cfg(test)]
mod exercises;
