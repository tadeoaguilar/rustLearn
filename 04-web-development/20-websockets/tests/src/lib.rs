//! Tests for 20-websockets.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m20_websockets_solution as sut;

#[cfg(feature = "mine")]
pub use m20_websockets as sut;

#[cfg(test)]
mod exercises;
