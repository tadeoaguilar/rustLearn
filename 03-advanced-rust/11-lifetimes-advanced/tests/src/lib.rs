//! Tests for 11-lifetimes-advanced.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m11_lifetimes_advanced_solution as sut;

#[cfg(feature = "mine")]
pub use m11_lifetimes_advanced as sut;

#[cfg(test)]
mod exercises;

// Compile-time checks: always on against the solution; opt-in against yours.
#[cfg(all(test, any(not(feature = "mine"), feature = "signatures")))]
mod signatures;
