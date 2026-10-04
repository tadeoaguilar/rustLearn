//! Tests for 01-getting-started.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test below is written
//! against `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m01_getting_started_solution as sut;

#[cfg(feature = "mine")]
pub use m01_getting_started as sut;

#[cfg(test)]
mod exercises;
