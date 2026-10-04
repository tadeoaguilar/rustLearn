//! Exercise 9: Shape Calculator.

use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    pub fn area(&self) -> f64 {
        todo!("Exercise 9")
    }

    /// The exercise says "for triangle, assume equilateral". But an
    /// equilateral triangle's height is fixed by its base (h = b·√3/2), so a
    /// `Triangle { base: 6.0, height: 8.0 }` can't be one. We assume
    /// *isosceles* instead -- the only triangle fully determined by a base and
    /// a height -- and each slanted side is sqrt((b/2)² + h²).
    pub fn perimeter(&self) -> f64 {
        todo!("Exercise 9")
    }

    /// Matching on `self` (a `&mut Shape`) gives `&mut f64` bindings, so we
    /// can write through them. Area grows by factor², perimeter by factor.
    pub fn scale(&mut self, factor: f64) {
        todo!("Exercise 9")
    }
}

pub fn total_area(shapes: &[Shape]) -> f64 {
    todo!("Exercise 9")
}

pub fn run() {
    todo!("Exercise 9")
}
