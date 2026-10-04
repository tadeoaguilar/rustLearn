//! Tests for 10-smart-pointers.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m10_smart_pointers_solution as sut;

#[cfg(feature = "mine")]
pub use m10_smart_pointers as sut;

#[cfg(test)]
mod exercises;
