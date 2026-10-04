//! Bonus Challenge: Shape System.
//!
//! `trait Drawable: Shape` is a *supertrait*: anything Drawable must also be a
//! Shape, so Drawable's methods (and its users) can call area()/perimeter().

use std::f64::consts::PI;
use std::fmt;

pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;

    fn name(&self) -> &'static str {
        "shape"
    }
}

pub trait Drawable: Shape {
    /// What `draw` prints. Returning a String makes it testable.
    fn render(&self) -> String;

    fn draw(&self) {
        println!("{}", self.render());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
    fn perimeter(&self) -> f64 {
        2.0 * PI * self.radius
    }
    fn name(&self) -> &'static str {
        "circle"
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
    fn name(&self) -> &'static str {
        "rectangle"
    }
}

impl Drawable for Circle {
    fn render(&self) -> String {
        format!("Drawing circle with radius {}", self.radius)
    }
}

impl Drawable for Rectangle {
    fn render(&self) -> String {
        format!("Drawing rectangle {}x{}", self.width, self.height)
    }
}

/// Static dispatch: every element is the *same* T.
pub fn total_area<T: Shape>(shapes: &[T]) -> f64 {
    shapes.iter().map(Shape::area).sum()
}

/// Dynamic dispatch: elements can be different types.
pub fn total_area_dyn(shapes: &[Box<dyn Drawable>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum() // supertrait method through dyn Drawable
}

pub fn draw_all(shapes: &[Box<dyn Drawable>]) -> Vec<String> {
    shapes.iter().map(|s| s.render()).collect()
}

/// Trait upcasting (stable since Rust 1.86): a `&dyn Drawable` converts to a
/// `&dyn Shape` because Shape is a supertrait.
pub fn as_shape(d: &dyn Drawable) -> &dyn Shape {
    d
}

/// Blanket implementation: *every* Shape gets a `describe` method, including
/// ones written later by other people.
pub trait Describe {
    fn describe(&self) -> String;
}

impl<T: Shape + ?Sized> Describe for T {
    fn describe(&self) -> String {
        format!(
            "{} with area {:.2} and perimeter {:.2}",
            self.name(),
            self.area(),
            self.perimeter()
        )
    }
}

/// A wrapper that gives any Shape a Display impl -- you can't `impl Display
/// for T` for a foreign trait on all T, but a newtype is fine.
pub struct Pretty<'a, S: Shape + ?Sized>(pub &'a S);

impl<S: Shape + ?Sized> fmt::Display for Pretty<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}: area {:.1}]", self.0.name(), self.0.area())
    }
}

pub fn run() {
    let circles = vec![Circle { radius: 1.0 }, Circle { radius: 2.0 }];
    println!("Total circle area: {:.4}", total_area(&circles));

    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 3.0 }),
        Box::new(Rectangle {
            width: 4.0,
            height: 5.0,
        }),
    ];
    for s in &shapes {
        s.draw();
    }
    println!("Total mixed area: {:.4}", total_area_dyn(&shapes));
    for s in &shapes {
        println!("{}  {}", s.describe(), Pretty(as_shape(s.as_ref())));
    }
}
