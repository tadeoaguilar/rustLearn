//! Exercise 2: Data Types.

use std::mem::size_of;

/// Size in bytes of every primitive the exercise asks about.
pub fn type_sizes() -> Vec<(&'static str, usize)> {
    todo!("Exercise 2")
}

/// A tuple groups values of *different* types; destructure it or use `.0`.
pub fn tuple_example() -> (i32, f64, char, &'static str) {
    todo!("Exercise 2")
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
    todo!("Exercise 2")
}

pub fn run() {
    // Integer types
    let small: i8 = 127;
    let big: i64 = 9_223_372_036_854_775_807;

    // Your code here: i16, i32, i128, isize, f32, f64, bool, a Unicode char...

    // Print sizes (use type_sizes() once you have written it)
    println!("i8 size: {} bytes", std::mem::size_of::<i8>());

    // Create a tuple with mixed types and destructure it

    // Create an array of 5 integers and access its elements

    todo!("Exercise 2: delete this line when you are done")
}
