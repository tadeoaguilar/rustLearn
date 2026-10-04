//! Exercise 1: Raw Pointers.

use std::ptr;

/// Swaps through raw pointers. Creating them is safe; using them isn't.
pub fn swap_via_pointers(a: &mut i32, b: &mut i32) {
    todo!("Exercise 1")
}

/// Sums by walking a pointer instead of indexing.
pub fn sum_with_pointer_arithmetic(slice: &[i32]) -> i32 {
    todo!("Exercise 1")
}

/// The Rust Book's example. The safe version
///
/// ```text
/// (&mut slice[..mid], &mut slice[mid..])
///       ^^^^^ error[E0499]: cannot borrow `*slice` as mutable more than once at a time
/// ```
///
/// is rejected because the borrow checker reasons about *whole values*: it
/// can't see that the two ranges don't overlap. We know they don't, so we
/// build both slices from a raw pointer.
pub fn my_split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
