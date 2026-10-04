//! Exercise 4: Basic Enums.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Task 1. (x, y) delta for one step.
pub fn move_player(direction: Direction) -> (i32, i32) {
    todo!("Exercise 4")
}

/// Applies a sequence of moves starting from the origin.
pub fn walk(directions: &[Direction]) -> (i32, i32) {
    todo!("Exercise 4")
}

/// Task 2. Each variant can carry different data: none, named fields, a
/// tuple. Writing this with structs would need four types and a trait.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    /// Returns the description instead of printing it, so it can be tested.
    pub fn describe(&self) -> String {
        todo!("Exercise 4")
    }

    pub fn call(&self) {
        todo!("Exercise 4")
    }
}

pub fn run() {
    todo!("Exercise 4")
}
