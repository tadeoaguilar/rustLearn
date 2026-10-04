//! Exercise 1: Vector Operations.

use std::collections::HashSet;
use std::hash::Hash;

/// Task 1a: remove duplicates, keeping the *first* occurrence and the order.
/// `HashSet::insert` returns false if the value was already there.
pub fn dedup_keep_order<T: Eq + Hash + Clone>(v: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    v.iter()
        .filter(|x| seen.insert((*x).clone()))
        .cloned()
        .collect()
}

/// Task 1b: if order doesn't matter, sort then `Vec::dedup` (which only
/// removes *adjacent* duplicates). In place, no extra allocation.
pub fn dedup_sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort_unstable();
    v.dedup();
    v
}

/// Task 2: split a Vec wherever `separator` appears, like `str::split`.
/// `[1, 0, 2, 3, 0, 4]` split on 0 -> `[[1], [2, 3], [4]]`.
pub fn split_on<T: PartialEq + Clone>(v: &[T], separator: &T) -> Vec<Vec<T>> {
    v.split(|x| x == separator).map(<[T]>::to_vec).collect()
}

/// Task 2, other reading: split at the *first* occurrence, value excluded.
pub fn split_at_value<T: PartialEq + Clone>(v: &[T], value: &T) -> Option<(Vec<T>, Vec<T>)> {
    let i = v.iter().position(|x| x == value)?;
    Some((v[..i].to_vec(), v[i + 1..].to_vec()))
}

/// Task 3: merge two sorted vectors in O(n + m) -- the merge step of merge
/// sort. (`a.extend(b); a.sort()` is O((n+m) log(n+m)) and ignores the fact
/// that the inputs are already sorted.)
pub fn merge_sorted<T: Ord + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            out.push(a[i].clone());
            i += 1;
        } else {
            out.push(b[j].clone());
            j += 1;
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
    out
}

#[allow(clippy::useless_vec)] // the exercise is about Vec, so Vec it is
pub fn run() {
    let mut v1: Vec<i32> = Vec::new();
    let v2 = vec![1, 2, 3, 4, 5];
    v1.push(1);
    v1.push(2);
    v1.extend([3, 4, 5]);
    let third = &v2[2];
    let third_option = v2.get(2);
    println!(
        "third = {third}, get(2) = {third_option:?}, get(10) = {:?}",
        v2.get(10)
    );
    for i in &mut v1 {
        *i *= 2;
    }
    println!("doubled in place: {v1:?}");

    let dupes = vec![3, 1, 3, 2, 1, 4];
    println!(
        "dedup_keep_order({dupes:?}) = {:?}",
        dedup_keep_order(&dupes)
    );
    println!(
        "dedup_sorted({dupes:?})     = {:?}",
        dedup_sorted(dupes.clone())
    );
    println!(
        "split_on([1,0,2,3,0,4], 0) = {:?}",
        split_on(&[1, 0, 2, 3, 0, 4], &0)
    );
    println!(
        "split_at_value([1,2,3,4], 3) = {:?}",
        split_at_value(&[1, 2, 3, 4], &3)
    );
    println!(
        "merge_sorted([1,4,9], [2,3,10]) = {:?}",
        merge_sorted(&[1, 4, 9], &[2, 3, 10])
    );
}
