//! Exercise 7: Result<T, E>.

use std::fmt;
use std::num::ParseIntError;

/// Task 1.
pub fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    s.trim().parse::<i32>()
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
        match self {
            MathError::DivisionByZero => write!(f, "cannot divide by zero"),
            MathError::NegativeSquareRoot => {
                write!(f, "cannot take the square root of a negative number")
            }
        }
    }
}

pub fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

pub fn sqrt(x: f64) -> Result<f64, MathError> {
    if x < 0.0 {
        Err(MathError::NegativeSquareRoot)
    } else {
        Ok(x.sqrt())
    }
}

/// Combining the two with `?`: the first error stops the computation and is
/// returned to the caller. sqrt(a / b).
pub fn sqrt_of_quotient(a: f64, b: f64) -> Result<f64, MathError> {
    let q = divide(a, b)?;
    sqrt(q)
}

pub fn run() {
    match parse_int("42") {
        Ok(n) => println!("Parsed: {n}"),
        Err(e) => println!("Error: {e}"),
    }
    match parse_int("forty-two") {
        Ok(n) => println!("Parsed: {n}"),
        Err(e) => println!("Error: {e}"),
    }
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {result}"),
        Err(MathError::DivisionByZero) => println!("Cannot divide by zero"),
        Err(e) => println!("Error: {e:?}"),
    }
    println!("sqrt_of_quotient(8, 2) = {:?}", sqrt_of_quotient(8.0, 2.0));
    println!("sqrt_of_quotient(8, 0) = {:?}", sqrt_of_quotient(8.0, 0.0));
    if let Err(e) = sqrt_of_quotient(-8.0, 2.0) {
        println!("sqrt_of_quotient(-8, 2) failed: {e}");
    }
}
