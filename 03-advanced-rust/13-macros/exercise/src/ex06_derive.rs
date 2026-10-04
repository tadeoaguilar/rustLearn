//! Exercise 6: A derive macro. Write `Describe` in ../derive/src/lib.rs.
//! Until it generates code, `Point::describe()` etc. don't exist.

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
    todo!("Exercise 6: print Point::describe() and friends")
}
