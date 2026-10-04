//! Exercise 3: Functions.

/// Returning the String keeps the function testable; `greet` prints it.
pub fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

pub fn greet(name: &str) {
    println!("{}", greeting(name));
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b // no semicolon: this expression is the return value
}

pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

/// Returning a tuple is how Rust returns "several values".
pub fn swap(a: i32, b: i32) -> (i32, i32) {
    (b, a)
}

/// Bonus: a function that takes another function.
///
/// `f: fn(i32) -> i32` accepts plain functions *and* closures that capture
/// nothing. Module 07 shows the more general `impl Fn(i32) -> i32`.
pub fn apply_twice(f: fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

pub fn run() {
    greet("Alice");
    assert_eq!(add(2, 3), 5);
    assert!(is_even(4));
    assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
    let (x, y) = swap(1, 2);
    assert_eq!((x, y), (2, 1));
    println!(
        "add(2, 3) = {}, is_even(4) = {}, 0C = {}F, swap(1, 2) = {:?}",
        add(2, 3),
        is_even(4),
        celsius_to_fahrenheit(0.0),
        swap(1, 2)
    );

    fn add_three(n: i32) -> i32 {
        n + 3
    }
    println!("apply_twice(add_three, 1) = {}", apply_twice(add_three, 1));
    println!("apply_twice(|n| n * n, 3) = {}", apply_twice(|n| n * n, 3));
}
