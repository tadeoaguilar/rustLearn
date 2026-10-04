//! Exercise 3: Methods and Associated Functions.
//!
//! The four kinds of function in an `impl` block, by their first parameter:
//!
//! | First parameter | Called as            | Meaning                         |
//! |-----------------|----------------------|---------------------------------|
//! | (none)          | `Rectangle::new(..)` | associated function, often `new` |
//! | `&self`         | `rect.area()`        | read only                       |
//! | `&mut self`     | `rect.scale(2)`      | modify in place                 |
//! | `self`          | `rect.to_square()`   | consume; caller can't use it after |

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn new(width: u32, height: u32) -> Self {
        todo!("Exercise 3")
    }

    pub fn square(size: u32) -> Self {
        todo!("Exercise 3")
    }

    pub fn area(&self) -> u32 {
        todo!("Exercise 3")
    }

    /// Strictly larger in both dimensions, as the exercise defines it.
    pub fn can_hold(&self, other: &Rectangle) -> bool {
        todo!("Exercise 3")
    }

    pub fn scale(&mut self, factor: u32) {
        todo!("Exercise 3")
    }

    pub fn to_square(self) -> Rectangle {
        todo!("Exercise 3")
    }

    pub fn is_square(&self) -> bool {
        todo!("Exercise 3")
    }
}

pub fn run() {
    todo!("Exercise 3")
}
