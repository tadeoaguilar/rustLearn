//! Exercise 2: Tuple Structs and Unit Structs.

/// Task 1. Same shape as Point, different type. Passing a Color where a
/// Point is expected is a compile error -- that is the point of "newtypes".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub i32, pub i32, pub i32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub i32, pub i32, pub i32);

/// Task 2. A unit struct holds no data and takes no space. Useful as a marker
/// or for implementing a trait on "nothing".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlwaysEqual;

/// Task 3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D(pub f64, pub f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D(pub f64, pub f64, pub f64);

/// sqrt(x² + y²). `hypot` computes the same thing without overflowing for
/// very large inputs.
pub fn distance_from_origin(point: Point2D) -> f64 {
    todo!("Exercise 2")
}

/// The same, for 3D -- not asked for, but Point3D was defined and unused.
pub fn distance_from_origin_3d(point: Point3D) -> f64 {
    todo!("Exercise 2")
}

pub fn run() {
    todo!("Exercise 2")
}
