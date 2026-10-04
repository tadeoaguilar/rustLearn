//! Tests for 05-error-handling.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m05_error_handling_solution as sut;

#[cfg(feature = "mine")]
pub use m05_error_handling as sut;

#[cfg(test)]
mod exercises;
