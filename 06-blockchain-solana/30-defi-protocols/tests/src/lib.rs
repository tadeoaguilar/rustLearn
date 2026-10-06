//! Tests for 30-defi-protocols.
//!
//! `sut` ("system under test") is either the reference solution or your own
//! exercise crate, chosen by the `mine` feature. Every test is written against
//! `sut`, so the same tests check both.

#[cfg(not(feature = "mine"))]
pub use m30_defi_protocols_solution as sut;

#[cfg(feature = "mine")]
pub use m30_defi_protocols as sut;

#[cfg(test)]
mod exercises;

// `--features sbf`: the same programs, compiled for the Solana VM.
#[cfg(all(test, feature = "sbf"))]
mod sbf;

/// Which build `./build-sbf.sh` made for the `sbf` tests.
#[cfg(feature = "sbf")]
pub const WHICH: &str = if cfg!(feature = "mine") {
    "mine"
} else {
    "solution"
};
