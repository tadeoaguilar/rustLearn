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
        todo!("Bonus")
    }
}

pub trait Drawable: Shape {
    /// What `draw` prints. Returning a String makes it testable.
    fn render(&self) -> String;

    fn draw(&self) {
        todo!("Bonus")
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
        todo!("Bonus")
    }
    fn perimeter(&self) -> f64 {
        todo!("Bonus")
    }
    fn name(&self) -> &'static str {
        todo!("Bonus")
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        todo!("Bonus")
    }
    fn perimeter(&self) -> f64 {
        todo!("Bonus")
    }
    fn name(&self) -> &'static str {
        todo!("Bonus")
    }
}

impl Drawable for Circle {
    fn render(&self) -> String {
        todo!("Bonus")
    }
}

impl Drawable for Rectangle {
    fn render(&self) -> String {
        todo!("Bonus")
    }
}

/// Static dispatch: every element is the *same* T.
pub fn total_area<T: Shape>(shapes: &[T]) -> f64 {
    todo!("Bonus")
}

/// Dynamic dispatch: elements can be different types.
pub fn total_area_dyn(shapes: &[Box<dyn Drawable>]) -> f64 {
    todo!("Bonus")
}

pub fn draw_all(shapes: &[Box<dyn Drawable>]) -> Vec<String> {
    todo!("Bonus")
}

/// Trait upcasting (stable since Rust 1.86): a `&dyn Drawable` converts to a
/// `&dyn Shape` because Shape is a supertrait.
pub fn as_shape(d: &dyn Drawable) -> &dyn Shape {
    todo!("Bonus")
}

/// Blanket implementation: *every* Shape gets a `describe` method, including
/// ones written later by other people.
pub trait Describe {
    fn describe(&self) -> String;
}

impl<T: Shape + ?Sized> Describe for T {
    fn describe(&self) -> String {
        todo!("Bonus")
    }
}

/// A wrapper that gives any Shape a Display impl -- you can't `impl Display
/// for T` for a foreign trait on all T, but a newtype is fine.
pub struct Pretty<'a, S: Shape + ?Sized>(pub &'a S);

impl<S: Shape + ?Sized> fmt::Display for Pretty<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Bonus")
    }
}

pub fn run() {
    todo!("Bonus")
}
