//! Tests for 14-unsafe-ffi.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

// Clippy warns when a name containing "unsafe" is aliased to one that doesn't
// (it could hide unsafety). Here it's just the module's name.
#![allow(clippy::unsafe_removed_from_name)]

#[cfg(not(feature = "mine"))]
pub use m14_unsafe_ffi_solution as sut;

#[cfg(feature = "mine")]
pub use m14_unsafe_ffi as sut;

#[cfg(test)]
mod exercises;
