//! Exercise 3: Iterator Basics.
//!
//! `iter()` yields `&T`, `iter_mut()` yields `&mut T`, `into_iter()` yields `T`.
//! That's why `filter` closures often see `&&i32`: filter passes a reference
//! to each item, and the items are already references.

/// Task 1.
pub fn sum_of_squares_of_evens(v: &[i32]) -> i32 {
    todo!("Exercise 3")
}

/// Task 2. `find` stops at the first match -- the rest of the (possibly
/// infinite) iterator is never touched.
pub fn first_divisible_by_3_and_5(v: &[i32]) -> Option<i32> {
    todo!("Exercise 3")
}

/// The same question over *all* positive integers: an infinite range is fine
/// because iterators are lazy.
pub fn first_positive_divisible_by(a: u32, b: u32) -> u32 {
    todo!("Exercise 3")
}

/// Task 3.
pub fn flatten<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    todo!("Exercise 3")
}

/// The basics from the exercise, returned so they can be checked.
pub struct Basics {
    pub sum: i32,
    pub product: i32,
    pub doubled: Vec<i32>,
    pub evens: Vec<i32>,
    pub even_squares: Vec<i32>,
    pub first_three: Vec<i32>,
    pub skip_two: Vec<i32>,
}

pub fn basics(v: &[i32]) -> Basics {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
