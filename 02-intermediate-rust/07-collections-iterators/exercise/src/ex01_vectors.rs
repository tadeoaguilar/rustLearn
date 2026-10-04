//! Exercise 1: Vector Operations.

use std::collections::HashSet;
use std::hash::Hash;

/// Task 1a: remove duplicates, keeping the *first* occurrence and the order.
/// `HashSet::insert` returns false if the value was already there.
pub fn dedup_keep_order<T: Eq + Hash + Clone>(v: &[T]) -> Vec<T> {
    todo!("Exercise 1")
}

/// Task 1b: if order doesn't matter, sort then `Vec::dedup` (which only
/// removes *adjacent* duplicates). In place, no extra allocation.
pub fn dedup_sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    todo!("Exercise 1")
}

/// Task 2: split a Vec wherever `separator` appears, like `str::split`.
/// `[1, 0, 2, 3, 0, 4]` split on 0 -> `[[1], [2, 3], [4]]`.
pub fn split_on<T: PartialEq + Clone>(v: &[T], separator: &T) -> Vec<Vec<T>> {
    todo!("Exercise 1")
}

/// Task 2, other reading: split at the *first* occurrence, value excluded.
pub fn split_at_value<T: PartialEq + Clone>(v: &[T], value: &T) -> Option<(Vec<T>, Vec<T>)> {
    todo!("Exercise 1")
}

/// Task 3: merge two sorted vectors in O(n + m) -- the merge step of merge
/// sort. (`a.extend(b); a.sort()` is O((n+m) log(n+m)) and ignores the fact
/// that the inputs are already sorted.)
pub fn merge_sorted<T: Ord + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
