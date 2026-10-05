//! Tests for 17-rest-apis.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m17_rest_apis_solution as sut;

#[cfg(feature = "mine")]
pub use m17_rest_apis as sut;

#[cfg(test)]
mod exercises;
