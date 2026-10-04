//! Exercise 5: Custom Iterators.
//!
//! Implement one method -- `next` -- and you get ~75 adaptors for free.

/// The exercise's Fibonacci. Changed: it returns None instead of overflowing.
/// The original computes `current + self.next` unconditionally, which panics
/// in a debug build at the 94th number.
#[derive(Debug, Clone)]
pub struct Fibonacci {
    curr: Option<u64>,
    next: Option<u64>,
}

impl Fibonacci {
    pub fn new() -> Self {
        todo!("Exercise 5")
    }
}

impl Default for Fibonacci {
    fn default() -> Self {
        todo!("Exercise 5")
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        todo!("Exercise 5")
    }
}

/// Task 1: a Range-like iterator with a step, counting up or down.
#[derive(Debug, Clone)]
pub struct StepRange {
    current: i64,
    end: i64,
    step: i64,
}

impl StepRange {
    /// Like `start..end` with a step. A negative step counts down.
    ///
    /// # Panics
    /// If `step` is zero (that range would never end).
    pub fn new(start: i64, end: i64, step: i64) -> Self {
        todo!("Exercise 5")
    }
}

impl Iterator for StepRange {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        todo!("Exercise 5")
    }
}

/// Task 2: cycles through a slice forever. It *borrows* the slice, so it needs
/// a lifetime: the iterator can't outlive the data. Yields `&'a T` -- items
/// that stay valid even after the iterator is gone.
#[derive(Debug, Clone)]
pub struct CycleSlice<'a, T> {
    items: &'a [T],
    index: usize,
}

impl<'a, T> CycleSlice<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        todo!("Exercise 5")
    }
}

impl<'a, T> Iterator for CycleSlice<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        todo!("Exercise 5")
    }
}

/// Task 3: an infinite prime generator. Keeps the primes found so far and
/// tests each candidate only against primes up to its square root.
#[derive(Debug, Default, Clone)]
pub struct Primes {
    found: Vec<u64>,
}

impl Primes {
    pub fn new() -> Self {
        todo!("Exercise 5")
    }
}

impl Iterator for Primes {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        todo!("Exercise 5")
    }
}

pub fn run() {
    todo!("Exercise 5")
}
