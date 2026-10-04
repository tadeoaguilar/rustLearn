//! Exercise 4: Operator Overloading.
//!
//! Operators are traits in `std::ops`: `a + b` is `Add::add(a, b)`.
//! Vector2D is Copy, so taking `self` by value costs nothing.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl Vector2D {
    pub const ZERO: Vector2D = Vector2D { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        todo!("Exercise 4")
    }

    pub fn magnitude(&self) -> f64 {
        todo!("Exercise 4")
    }

    pub fn dot(self, other: Vector2D) -> f64 {
        todo!("Exercise 4")
    }
}

impl Add for Vector2D {
    type Output = Vector2D;
    fn add(self, other: Vector2D) -> Vector2D {
        todo!("Exercise 4")
    }
}

impl Sub for Vector2D {
    type Output = Vector2D;
    fn sub(self, other: Vector2D) -> Vector2D {
        todo!("Exercise 4")
    }
}

/// `v * 2.0`: the right-hand side type is the trait's type parameter.
impl Mul<f64> for Vector2D {
    type Output = Vector2D;
    fn mul(self, scalar: f64) -> Vector2D {
        todo!("Exercise 4")
    }
}

/// `2.0 * v`: implementing a std trait for a std type (f64) is allowed here
/// because *our* type appears in it. That is the "orphan rule".
impl Mul<Vector2D> for f64 {
    type Output = Vector2D;
    fn mul(self, v: Vector2D) -> Vector2D {
        todo!("Exercise 4")
    }
}

/// `-v`
impl Neg for Vector2D {
    type Output = Vector2D;
    fn neg(self) -> Vector2D {
        todo!("Exercise 4")
    }
}

/// `v += w`
impl AddAssign for Vector2D {
    fn add_assign(&mut self, other: Vector2D) {
        todo!("Exercise 4")
    }
}

/// Lets `iter.sum::<Vector2D>()` work.
impl Sum for Vector2D {
    fn sum<I: Iterator<Item = Vector2D>>(iter: I) -> Self {
        todo!("Exercise 4")
    }
}

impl fmt::Display for Vector2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 4")
    }
}

pub fn run() {
    todo!("Exercise 4")
}
