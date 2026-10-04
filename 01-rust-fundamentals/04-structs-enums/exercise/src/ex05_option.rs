//! Exercise 5: Option<T>.
//!
//! Rust has no null. A value that might be missing has type `Option<T>`, and
//! the compiler won't let you use the `T` inside until you have checked.

/// Task 1.
pub fn divide(a: f64, b: f64) -> Option<f64> {
    todo!("Exercise 5")
}

pub fn expensive_computation() -> i32 {
    todo!("Exercise 5")
}

/// Task 2: the Option methods from the exercise, each result returned so the
/// tests can check them. Read the comments, not just the code.
///
/// With `some_number = Some(5)` and `no_number: Option<i32> = None`, return:
///
/// ```text
/// [ Some(some_number.unwrap()),
///   Some(no_number.unwrap_or(0)),
///   Some(no_number.unwrap_or_else(expensive_computation)),
///   some_number.map(|n| n * 2),
///   some_number.and_then(|n| if n > 3 { Some(n * 2) } else { None }),
///   some_number.filter(|n| n % 2 == 0) ]
/// ```
pub fn option_methods() -> [Option<i32>; 6] {
    todo!("Exercise 5")
}

/// Task 3. Iterator adaptors already return Option: `find` and `position`.
pub fn find_first_even(numbers: &[i32]) -> Option<i32> {
    todo!("Exercise 5")
}

pub fn find_position<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    todo!("Exercise 5")
}

/// The same as `find_position`, written with a plain loop, for comparison.
pub fn find_position_loop<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    todo!("Exercise 5")
}

pub fn run() {
    todo!("Exercise 5")
}
