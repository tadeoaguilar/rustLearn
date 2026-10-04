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
    let Point2D(x, y) = point; // destructure a tuple struct
    x.hypot(y)
}

/// The same, for 3D -- not asked for, but Point3D was defined and unused.
pub fn distance_from_origin_3d(point: Point3D) -> f64 {
    let Point3D(x, y, z) = point;
    (x * x + y * y + z * z).sqrt()
}

pub fn run() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
    println!("Red: {}, Green: {}, Blue: {}", black.0, black.1, black.2);
    println!("origin = {origin:?}");
    // let p: Point = black; // error[E0308]: mismatched types, expected `Point`, found `Color`

    let subject = AlwaysEqual;
    println!(
        "{subject:?} takes {} bytes",
        std::mem::size_of::<AlwaysEqual>()
    );

    println!(
        "distance (3, 4) = {}",
        distance_from_origin(Point2D(3.0, 4.0))
    );
    println!(
        "distance (1, 2, 2) = {}",
        distance_from_origin_3d(Point3D(1.0, 2.0, 2.0))
    );
}
