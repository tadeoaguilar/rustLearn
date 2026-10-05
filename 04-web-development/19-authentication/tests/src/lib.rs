//! Tests for 19-authentication.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m19_authentication_solution as sut;

#[cfg(feature = "mine")]
pub use m19_authentication as sut;

#[cfg(test)]
mod exercises;
