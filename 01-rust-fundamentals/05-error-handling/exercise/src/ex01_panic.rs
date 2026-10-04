//! Exercise 1: Panic Basics.
//!
//! `run()` uses `std::panic::catch_unwind` so the demo can show several panics
//! without the program dying at the first one. Don't use catch_unwind for
//! normal error handling -- it exists for things like keeping a thread pool
//! alive when one task panics.

/// Task 2 as written: an unrealistic age is treated as a *bug in the caller*.
pub fn set_age(age: i32) -> i32 {
    todo!("Exercise 1")
}

/// The same check as a recoverable error -- the right choice when the age
/// comes from user input, which can be wrong without anyone having a bug.
pub fn try_set_age(age: i32) -> Result<i32, String> {
    todo!("Exercise 1")
}

/// Task 3: `v[i]` panics when out of bounds; `v.get(i)` returns an Option.
pub fn element_at(v: &[i32], index: usize) -> Option<i32> {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
