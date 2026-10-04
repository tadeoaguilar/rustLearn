//! Exercise 6: A derive macro -- see `derive/src/lib.rs` for `Describe`.

use crate::Describe;

#[derive(Describe)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Describe)]
pub struct Meters(pub f64);

#[derive(Describe)]
pub struct Marker;

#[derive(Describe)]
pub enum Shape {
    Circle(f64),
    Square { side: f64 },
    Empty,
}

#[derive(Describe)]
pub struct Wrapper<T: Clone> {
    pub inner: T,
    pub history: Vec<T>,
}

#[derive(Describe)]
pub struct Profile {
    pub name: String,
    pub tags: Vec<String>,
    pub age: Option<u8>,
    pub motto: &'static str,
}

pub fn run() {
    println!("{}", Point::describe());
    println!("  field_names: {:?}", Point::field_names());
    println!("{}", Meters::describe());
    println!("{}", Marker::describe());
    println!("{}", Shape::describe());
    println!("{}", Wrapper::<i32>::describe());
    println!("{}", Profile::describe());
}
