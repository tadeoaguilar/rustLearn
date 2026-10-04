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
    match direction {
        Direction::North => (0, 1),
        Direction::South => (0, -1),
        Direction::East => (1, 0),
        Direction::West => (-1, 0),
    }
}

/// Applies a sequence of moves starting from the origin.
pub fn walk(directions: &[Direction]) -> (i32, i32) {
    directions.iter().fold((0, 0), |(x, y), &d| {
        let (dx, dy) = move_player(d);
        (x + dx, y + dy)
    })
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
        match self {
            Message::Quit => "Quit".to_string(),
            Message::Move { x, y } => format!("Move to ({x}, {y})"),
            Message::Write(text) => format!("Write: {text}"),
            Message::ChangeColor(r, g, b) => format!("Change color to ({r}, {g}, {b})"),
        }
    }

    pub fn call(&self) {
        println!("{}", self.describe());
    }
}

pub fn run() {
    use Direction::{East, North};
    println!("North -> {:?}", move_player(North));
    println!("walk [N, N, E] -> {:?}", walk(&[North, North, East]));

    let messages = vec![
        Message::Quit,
        Message::Move { x: 10, y: 20 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(255, 0, 0),
    ];
    for msg in messages {
        msg.call();
    }
}
