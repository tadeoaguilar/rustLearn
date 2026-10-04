//! Exercise 2: Data Types.

use std::mem::size_of;

/// Size in bytes of every primitive the exercise asks about.
pub fn type_sizes() -> Vec<(&'static str, usize)> {
    vec![
        ("i8", size_of::<i8>()),
        ("i16", size_of::<i16>()),
        ("i32", size_of::<i32>()),
        ("i64", size_of::<i64>()),
        ("i128", size_of::<i128>()),
        ("isize", size_of::<isize>()), // pointer-sized: 8 on 64-bit machines
        ("f32", size_of::<f32>()),
        ("f64", size_of::<f64>()),
        ("bool", size_of::<bool>()),
        ("char", size_of::<char>()), // 4 bytes: any Unicode scalar value
    ]
}

/// A tuple groups values of *different* types; destructure it or use `.0`.
pub fn tuple_example() -> (i32, f64, char, &'static str) {
    (500, 6.4, '🦀', "crab")
}

/// An array has a fixed length that is part of its type: `[i32; 5]`.
pub fn array_example() -> [i32; 5] {
    [1, 2, 3, 4, 5]
}

/// Bonus: integer overflow.
///
/// `a + b` with i8 values 127 + 1 *panics* in a debug build ("attempt to add
/// with overflow") and silently wraps to -128 in a release build. When
/// overflow is possible, say what you want explicitly:
pub fn overflow_strategies(a: i8, b: i8) -> (i8, Option<i8>, i8, (i8, bool)) {
    (
        a.wrapping_add(b),    // wrap around: 127 + 1 = -128
        a.checked_add(b),     // None on overflow
        a.saturating_add(b),  // clamp: 127 + 1 = 127
        a.overflowing_add(b), // wrapped value + "did it overflow?"
    )
}

pub fn run() {
    let small: i8 = 127;
    let big: i64 = 9_223_372_036_854_775_807;
    let huge: i128 = 170_141_183_460_469_231_731_687_303_715_884_105_727;
    let pi: f64 = std::f64::consts::PI;
    let e: f32 = std::f32::consts::E;
    let is_rust_fun = true;
    let heart = '❤';
    println!("i8 max {small}, i64 max {big}\ni128 max {huge}");
    println!("f64 {pi}, f32 {e}, bool {is_rust_fun}, char {heart}");

    for (name, bytes) in type_sizes() {
        println!("{name:>5} size: {bytes} bytes");
    }

    let tup = tuple_example();
    let (number, float, crab, word) = tup; // destructuring
    println!(
        "tuple: {tup:?} -> {number}, {float}, {crab}, {word}; tup.2 = {}",
        tup.2
    );

    let arr = array_example();
    println!(
        "array: {arr:?}, first {}, last {}, len {}",
        arr[0],
        arr[arr.len() - 1],
        arr.len()
    );
    let zeros = [0u8; 3]; // [value; count]
    println!("[0u8; 3] = {zeros:?}");
    // arr[10] would panic at runtime: index out of bounds. `arr.get(10)` returns None.
    println!("arr.get(10) = {:?}", arr.get(10));

    let (wrapped, checked, saturated, overflowing) = overflow_strategies(127, 1);
    println!(
        "127i8 + 1: wrapping {wrapped}, checked {checked:?}, saturating {saturated}, overflowing {overflowing:?}"
    );
}
