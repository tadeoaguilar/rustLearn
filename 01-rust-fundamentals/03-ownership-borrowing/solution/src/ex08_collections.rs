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
    v.first().map(String::as_str)
}

/// Iterating `&v` borrows each element; `v` is still usable afterwards.
pub fn total_len(v: &[String]) -> usize {
    let mut total = 0;
    for s in v {
        total += s.len();
    }
    total
}

/// Iterating `v` by value moves each String out; the Vec is consumed.
pub fn into_upper(v: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(v.len());
    for s in v {
        out.push(s.to_uppercase()); // s is owned here
    }
    out
}

/// Task 2: `&mut` iteration hands out `&mut i32`; `*n` writes through it.
pub fn double_in_place(numbers: &mut [i32]) {
    for n in numbers.iter_mut() {
        *n *= 2;
    }
}

/// Task 3: `retain` keeps the elements for which the closure returns true.
pub fn remove_negatives(numbers: &mut Vec<i32>) {
    numbers.retain(|&n| n >= 0);
}

/// Task 3, the other hint. `drain_filter` was never stabilised; it shipped in
/// Rust 1.87 as `extract_if`. Unlike `retain`, it *gives you* the removed
/// elements.
pub fn take_negatives(numbers: &mut Vec<i32>) -> Vec<i32> {
    numbers.extract_if(.., |n| *n < 0).collect()
}

pub fn run() {
    let v = vec![String::from("hello"), String::from("world")];
    println!("first = {:?}, total_len = {}", first(&v), total_len(&v));
    println!("still have v: {v:?}");
    println!("into_upper(v) = {:?}", into_upper(v));
    // println!("{v:?}"); // error[E0382]: borrow of moved value: `v`

    let mut numbers = vec![1, 2, 3, 4, 5];
    double_in_place(&mut numbers);
    println!("doubled: {numbers:?}");

    let mut nums = vec![1, -2, 3, -4, 5];
    remove_negatives(&mut nums);
    println!("retain: {nums:?}");

    let mut nums = vec![1, -2, 3, -4, 5];
    let removed = take_negatives(&mut nums);
    println!("extract_if: kept {nums:?}, removed {removed:?}");
}
