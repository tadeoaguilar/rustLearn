//! Tests for 38-proc-macros. See Cargo.toml for how the features select exercises.

#[cfg(feature = "mine")]
pub use m38_proc_macros as sut;
#[cfg(not(feature = "mine"))]
pub use m38_proc_macros_solution as sut;

#[cfg(feature = "mine")]
pub use m38_proc_macros_core as core_sut;
#[cfg(not(feature = "mine"))]
pub use m38_proc_macros_core_solution as core_sut;

// The expansion functions, called directly: always compiled.
#[cfg(test)]
mod expansion;

// The macros used on real types: against yours, only the ones switched on.
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
