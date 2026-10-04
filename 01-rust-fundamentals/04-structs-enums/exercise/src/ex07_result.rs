//! Exercise 7: Result<T, E>.

use std::fmt;
use std::num::ParseIntError;

/// Task 1.
pub fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    todo!("Exercise 7")
}

/// Task 2: your own error enum. Deriving PartialEq lets tests compare errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
}

/// Display is what users see; Debug is for developers. Module 05 builds on this.
impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 7")
    }
}

pub fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    todo!("Exercise 7")
}

pub fn sqrt(x: f64) -> Result<f64, MathError> {
    todo!("Exercise 7")
}

/// Combining the two with `?`: the first error stops the computation and is
/// returned to the caller. sqrt(a / b).
pub fn sqrt_of_quotient(a: f64, b: f64) -> Result<f64, MathError> {
    todo!("Exercise 7")
}

pub fn run() {
    todo!("Exercise 7")
}
