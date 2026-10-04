//! Exercise 4: Advanced Iterators.

/// The exercise's examples, returned for testing.
#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedBasics {
    pub sum: i32,
    pub running_sum: Vec<i32>,
    pub combined: Vec<(i32, &'static str)>,
    pub flat: Vec<i32>,
}

pub fn advanced_basics() -> AdvancedBasics {
    todo!("Exercise 4")
}

/// Task 1: Fibonacci with `std::iter::successors`, which repeatedly applies a
/// function to the previous item until it returns None.
///
/// The state is (current, next). `next` is an Option: `checked_add` turns it
/// into None when the following number would overflow a u64, and the
/// iterator ends one step later -- after yielding the largest u64 Fibonacci
/// number (F93), not before it.
pub fn fibonacci() -> impl Iterator<Item = u64> {
    // A bare `todo!()` doesn't compile here: `impl Iterator` needs *some*
    // concrete iterator type behind it. Replace both lines.
    todo!("Exercise 4");
    std::iter::empty()
}

/// Task 2: every (x, y) pair. `flat_map` + `map` is a nested loop as an iterator.
pub fn cartesian_product<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<(A, B)> {
    todo!("Exercise 4")
}

/// Task 3: one pass, two outputs.
pub fn partition_even_odd(v: &[i32]) -> (Vec<i32>, Vec<i32>) {
    todo!("Exercise 4")
}

pub fn run() {
    todo!("Exercise 4")
}
