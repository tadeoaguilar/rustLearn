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
        match self {
            Shape::Circle { radius } => PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }

    /// The exercise says "for triangle, assume equilateral". But an
    /// equilateral triangle's height is fixed by its base (h = b·√3/2), so a
    /// `Triangle { base: 6.0, height: 8.0 }` can't be one. We assume
    /// *isosceles* instead -- the only triangle fully determined by a base and
    /// a height -- and each slanted side is sqrt((b/2)² + h²).
    pub fn perimeter(&self) -> f64 {
        match self {
            Shape::Circle { radius } => 2.0 * PI * radius,
            Shape::Rectangle { width, height } => 2.0 * (width + height),
            Shape::Triangle { base, height } => {
                let side = (base / 2.0).hypot(*height);
                base + 2.0 * side
            }
        }
    }

    /// Matching on `self` (a `&mut Shape`) gives `&mut f64` bindings, so we
    /// can write through them. Area grows by factor², perimeter by factor.
    pub fn scale(&mut self, factor: f64) {
        match self {
            Shape::Circle { radius } => *radius *= factor,
            Shape::Rectangle { width, height }
            | Shape::Triangle {
                base: width,
                height,
            } => {
                *width *= factor;
                *height *= factor;
            }
        }
    }
}

pub fn total_area(shapes: &[Shape]) -> f64 {
    shapes.iter().map(Shape::area).sum()
}

pub fn run() {
    let mut shapes = vec![
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle {
            width: 10.0,
            height: 20.0,
        },
        Shape::Triangle {
            base: 6.0,
            height: 8.0,
        },
    ];
    for shape in &shapes {
        println!(
            "{shape:?} has area: {:.2}, perimeter: {:.2}",
            shape.area(),
            shape.perimeter()
        );
    }
    println!("Total area: {:.2}", total_area(&shapes));

    for shape in &mut shapes {
        shape.scale(2.0);
    }
    println!(
        "Total area after scaling x2: {:.2} (x4)",
        total_area(&shapes)
    );
}
