//! Exercise 2: Result Basics.

/// Task 1: a `String` error is quick to write, but callers can only print it
/// -- they can't `match` on *which* error happened.
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
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
    if b == 0 {
        return Err(DivisionError::DivideByZero);
    }
    a.checked_div(b).ok_or(DivisionError::Overflow)
}

pub fn run() {
    for (a, b) in [(10.0, 2.0), (10.0, 0.0)] {
        match divide(a, b) {
            Ok(result) => println!("divide({a}, {b}): Result: {result}"),
            Err(e) => println!("divide({a}, {b}): Error: {e}"),
        }
    }
    for (a, b) in [(10, 3), (1, 0), (i32::MIN, -1)] {
        match safe_divide(a, b) {
            Ok(v) => println!("safe_divide({a}, {b}) = {v}"),
            Err(DivisionError::DivideByZero) => {
                println!("safe_divide({a}, {b}): cannot divide by zero")
            }
            Err(DivisionError::Overflow) => println!("safe_divide({a}, {b}): overflow"),
        }
    }
}
