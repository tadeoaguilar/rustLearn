//! Exercise 1: Your First `macro_rules!`.  -- see exercises.md
//!
//! Write these, each with #[macro_export]:
//!
//!     square!(x)                  x * x, evaluating x only ONCE
//!     square_naive!(x)            $x * $x -- the buggy version, for comparison
//!     max!(a), max!(a, b, ...)    the largest; recursive
//!     hashmap!{ k => v, ... }     a HashMap; trailing comma allowed; hashmap!{} is empty
//!     vec_of_strings![a, b]       Vec<String> via ToString
//!
//! The helpers below count calls, so you can show the double evaluation.

thread_local! {
    static CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Returns 3 and counts how often it was called.
pub fn next_value() -> i32 {
    CALLS.with(|c| c.set(c.get() + 1));
    3
}

pub fn calls() -> u32 {
    CALLS.with(|c| c.get())
}

pub fn reset_calls() {
    CALLS.with(|c| c.set(0));
}

// TODO Exercise 1: your macros here.

pub fn run() {
    todo!("Exercise 1: call your macros and print the results")
}
