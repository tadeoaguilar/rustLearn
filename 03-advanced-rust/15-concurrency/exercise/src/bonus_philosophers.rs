//! Bonus: Dining Philosophers.
//!
//! Naive: philosopher i picks up fork i, then fork i+1. If all five pick up
//! their left fork at the same moment, each waits forever for the right one --
//! a cycle of waiting, i.e. deadlock.
//!
//! Fix: resource ordering. Everyone picks up the *lower-numbered* fork first.
//! Philosopher 4 (forks 4 and 0) now reaches for fork 0 first, which breaks
//! the cycle: at least one philosopher can always get both forks.

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub fn dine(philosophers: usize, meals: usize) -> Vec<usize> {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
