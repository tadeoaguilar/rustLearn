//! Exercise 1: Basic Async Functions.

use std::num::ParseIntError;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, sleep};

pub async fn say_hello() -> &'static str {
    todo!("Exercise 1")
}

/// `tokio::time::sleep`, never `std::thread::sleep`: the std version blocks
/// the whole worker thread, and every other task scheduled on it stalls.
pub async fn say_world() -> &'static str {
    todo!("Exercise 1")
}

/// Task 1: async functions that return values.
pub async fn add_async(a: i32, b: i32) -> i32 {
    todo!("Exercise 1")
}

/// Task 2: chaining -- each `.await` finishes before the next line runs.
pub async fn hello_world() -> String {
    todo!("Exercise 1")
}

/// Task 3: errors. `?` works in async fns exactly as in sync ones.
pub async fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    todo!("Exercise 1")
}

pub async fn sum_strings(a: &str, b: &str) -> Result<i32, ParseIntError> {
    todo!("Exercise 1")
}

/// Futures are lazy. Creating one runs *none* of its body; only awaiting
/// does. Returns (ran before await, ran after await) -- expect (false, true).
pub async fn futures_are_lazy() -> (bool, bool) {
    todo!("Exercise 1")
}

pub async fn run() {
    todo!("Exercise 1")
}
