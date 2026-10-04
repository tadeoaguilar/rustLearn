//! Tests for 13-macros. See Cargo.toml for how the features select exercises.

#[cfg(not(feature = "mine"))]
pub use m13_macros_solution as sut;

#[cfg(feature = "mine")]
pub use m13_macros as sut;

// Against the solution every module compiles; against yours, only the ones
// you've switched on.
#[cfg(all(test, any(not(feature = "mine"), feature = "bonus")))]
mod bonus;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex1")))]
mod ex1;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex2")))]
mod ex2;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex3")))]
mod ex3;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex4")))]
mod ex4;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex5")))]
mod ex5;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex6")))]
mod ex6;
#[cfg(all(test, any(not(feature = "mine"), feature = "ex7")))]
mod ex7;
