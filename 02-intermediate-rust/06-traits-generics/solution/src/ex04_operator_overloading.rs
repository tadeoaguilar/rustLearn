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
        Vector2D { x, y }
    }

    pub fn magnitude(&self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn dot(self, other: Vector2D) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vector2D {
    type Output = Vector2D;
    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vector2D {
    type Output = Vector2D;
    fn sub(self, other: Vector2D) -> Vector2D {
        Vector2D::new(self.x - other.x, self.y - other.y)
    }
}

/// `v * 2.0`: the right-hand side type is the trait's type parameter.
impl Mul<f64> for Vector2D {
    type Output = Vector2D;
    fn mul(self, scalar: f64) -> Vector2D {
        Vector2D::new(self.x * scalar, self.y * scalar)
    }
}

/// `2.0 * v`: implementing a std trait for a std type (f64) is allowed here
/// because *our* type appears in it. That is the "orphan rule".
impl Mul<Vector2D> for f64 {
    type Output = Vector2D;
    fn mul(self, v: Vector2D) -> Vector2D {
        v * self
    }
}

/// `-v`
impl Neg for Vector2D {
    type Output = Vector2D;
    fn neg(self) -> Vector2D {
        Vector2D::new(-self.x, -self.y)
    }
}

/// `v += w`
impl AddAssign for Vector2D {
    fn add_assign(&mut self, other: Vector2D) {
        self.x += other.x;
        self.y += other.y;
    }
}

/// Lets `iter.sum::<Vector2D>()` work.
impl Sum for Vector2D {
    fn sum<I: Iterator<Item = Vector2D>>(iter: I) -> Self {
        iter.fold(Vector2D::ZERO, Add::add)
    }
}

impl fmt::Display for Vector2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub fn run() {
    let v1 = Vector2D::new(1.0, 2.0);
    let v2 = Vector2D::new(3.0, 4.0);
    let v3 = v1 + v2;
    println!("{v1} + {v2} = {v3}");
    println!("{v2} - {v1} = {}", v2 - v1);
    println!("{v1} * 3 = {}  and  3 * {v1} = {}", v1 * 3.0, 3.0 * v1);
    println!("-{v1} = {}", -v1);
    println!("|{v2}| = {}", v2.magnitude());
    let mut acc = Vector2D::ZERO;
    acc += v1;
    acc += v2;
    println!("+= twice: {acc}");
    println!("sum: {}", [v1, v2, v3].into_iter().sum::<Vector2D>());
}
