//! Exercise 3: Functions.

/// Returning the String keeps the function testable; `greet` prints it.
pub fn greeting(name: &str) -> String {
    todo!("Exercise 3")
}

pub fn greet(name: &str) {
    todo!("Exercise 3")
}

pub fn add(a: i32, b: i32) -> i32 {
    todo!("Exercise 3")
}

pub fn is_even(n: i32) -> bool {
    todo!("Exercise 3")
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    todo!("Exercise 3")
}

/// Returning a tuple is how Rust returns "several values".
pub fn swap(a: i32, b: i32) -> (i32, i32) {
    todo!("Exercise 3")
}

/// Bonus: a function that takes another function.
///
/// `f: fn(i32) -> i32` accepts plain functions *and* closures that capture
/// nothing. Module 07 shows the more general `impl Fn(i32) -> i32`.
pub fn apply_twice(f: fn(i32) -> i32, x: i32) -> i32 {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
