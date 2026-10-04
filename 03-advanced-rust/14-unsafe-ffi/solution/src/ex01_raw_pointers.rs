//! Exercise 1: Raw Pointers.

use std::ptr;

/// Swaps through raw pointers. Creating them is safe; using them isn't.
pub fn swap_via_pointers(a: &mut i32, b: &mut i32) {
    let pa: *mut i32 = a;
    let pb = &raw mut *b; // the 2024 syntax for "a raw pointer to this place"
    // SAFETY: both pointers come from live `&mut` references, so they are
    // non-null, aligned and valid for reads and writes. `&mut a` and `&mut b`
    // can't alias, and ptr::swap handles overlap anyway.
    unsafe { ptr::swap(pa, pb) };
}

/// Sums by walking a pointer instead of indexing.
pub fn sum_with_pointer_arithmetic(slice: &[i32]) -> i32 {
    let start = slice.as_ptr();
    let mut total = 0;
    for i in 0..slice.len() {
        // SAFETY: i < slice.len(), so start.add(i) stays inside the slice's
        // allocation and points to an initialised i32.
        total += unsafe { *start.add(i) };
    }
    total
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
    let len = slice.len();
    // The safety of everything below depends on this check. It's what makes
    // this a *safe* function: no input can make the unsafe code misbehave.
    assert!(mid <= len, "mid ({mid}) > len ({len})");
    let ptr = slice.as_mut_ptr();
    // SAFETY: [0, mid) and [mid, len) are in bounds (checked above) and
    // don't overlap, so handing out a `&mut` to each never aliases. Both
    // live no longer than the input borrow, which the signature enforces.
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

pub fn run() {
    let (mut a, mut b) = (1, 2);
    swap_via_pointers(&mut a, &mut b);
    println!("after swap: a = {a}, b = {b}");
    println!(
        "sum via pointer arithmetic: {}",
        sum_with_pointer_arithmetic(&[1, 2, 3, 4])
    );
    let mut v = [1, 2, 3, 4, 5];
    let (left, right) = my_split_at_mut(&mut v, 2);
    left[0] = 10;
    right[0] = 30;
    println!("my_split_at_mut, then writes to both halves: {v:?}");
}
