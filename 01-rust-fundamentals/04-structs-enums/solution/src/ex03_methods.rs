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
        Rectangle { width, height }
    }

    pub fn square(size: u32) -> Self {
        Rectangle {
            width: size,
            height: size,
        }
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    /// Strictly larger in both dimensions, as the exercise defines it.
    pub fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    pub fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }

    pub fn to_square(self) -> Rectangle {
        let size = self.width.max(self.height);
        Rectangle::square(size)
    }

    pub fn is_square(&self) -> bool {
        self.width == self.height
    }
}

pub fn run() {
    let rect1 = Rectangle::new(30, 50);
    let rect2 = Rectangle::square(25);
    println!("Area: {}", rect1.area());
    println!("Can hold: {}", rect1.can_hold(&rect2));

    let mut rect3 = Rectangle::new(10, 20);
    rect3.scale(2);
    // exercises.md asserts 400 here. Scaling *both* sides by 2 gives 20 x 40,
    // so the area grows by 2² = 4: 200 -> 800.
    assert_eq!(rect3.area(), 800);
    println!("scaled: {rect3:?}, area {}", rect3.area());

    let square = rect3.to_square();
    println!("to_square: {square:?}");
    // println!("{rect3:?}"); // error[E0382]: borrow of moved value: `rect3`
}
