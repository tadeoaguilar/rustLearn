//! Acceptance tests for 09-testing -- the answer key.
//!
//! Each test is named after the bug it catches. Against the solution they
//! all pass; against the untouched exercise crate exactly six fail.
//! Write your own tests first, then use these to check you found everything.

#[cfg(not(feature = "mine"))]
pub use m09_testing_solution as sut;

#[cfg(feature = "mine")]
pub use m09_testing as sut;

#[cfg(test)]
mod exercises;
