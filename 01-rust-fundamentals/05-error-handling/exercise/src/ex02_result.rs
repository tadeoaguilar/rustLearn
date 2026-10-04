//! Exercise 2: Result Basics.

/// Task 1: a `String` error is quick to write, but callers can only print it
/// -- they can't `match` on *which* error happened.
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    todo!("Exercise 2")
}

/// Task 2: an enum error. Callers can match on each case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivisionError {
    DivideByZero,
    Overflow,
}

/// Integer division overflows in exactly one case: i32::MIN / -1, because
/// +2147483648 doesn't fit in an i32. `checked_div` returns None for it (and
/// for b == 0, but we've already handled that with a more specific error).
pub fn safe_divide(a: i32, b: i32) -> Result<i32, DivisionError> {
    todo!("Exercise 2")
}

pub fn run() {
    todo!("Exercise 2")
}
