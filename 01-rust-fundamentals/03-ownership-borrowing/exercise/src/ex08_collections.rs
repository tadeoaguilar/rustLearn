//! Exercise 8: Ownership in Collections.

/// Task 1: indexing a Vec gives you a place, and you can't *move* out of it:
///
/// ```text
/// let first = v[0];
///             ^^^^ error[E0507]: cannot move out of index of `Vec<String>`
///                  help: consider borrowing here: `&v[0]`
/// ```
///
/// Moving it would leave a hole in the vector. Borrow instead.
pub fn first(v: &[String]) -> Option<&str> {
    todo!("Exercise 8")
}

/// Iterating `&v` borrows each element; `v` is still usable afterwards.
pub fn total_len(v: &[String]) -> usize {
    todo!("Exercise 8")
}

/// Iterating `v` by value moves each String out; the Vec is consumed.
pub fn into_upper(v: Vec<String>) -> Vec<String> {
    todo!("Exercise 8")
}

/// Task 2: `&mut` iteration hands out `&mut i32`; `*n` writes through it.
pub fn double_in_place(numbers: &mut [i32]) {
    todo!("Exercise 8")
}

/// Task 3: `retain` keeps the elements for which the closure returns true.
pub fn remove_negatives(numbers: &mut Vec<i32>) {
    todo!("Exercise 8")
}

/// Task 3, the other hint. `drain_filter` was never stabilised; it shipped in
/// Rust 1.87 as `extract_if`. Unlike `retain`, it *gives you* the removed
/// elements.
pub fn take_negatives(numbers: &mut Vec<i32>) -> Vec<i32> {
    todo!("Exercise 8")
}

pub fn run() {
    todo!("Exercise 8")
}
