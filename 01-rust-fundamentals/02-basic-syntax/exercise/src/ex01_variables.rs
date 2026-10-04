//! Exercise 1: Variables and Mutability.

/// A constant: always annotated, always known at compile time, usable anywhere
/// (even at module level), and inlined at every use.
pub const MAX_POINTS: u32 = 100_000;

pub fn run() {
    // Create an immutable variable x with value 5

    // Try to change x (this should error -- read the message, then comment it out)
    // x = 6;

    // Create a mutable variable y with value 10

    // Change y to 20

    // Shadow x with a new value of different type

    // Create a variable with explicit type annotation

    todo!("Exercise 1: delete this line when the code above prints something")
}

/// The classic shadowing example from the Rust Book: the text `spaces` is
/// replaced by the number `spaces`, without inventing `spaces_str`/`spaces_num`.
///
/// With `let mut spaces = "   "; spaces = spaces.len();` you would get
/// error[E0308]: mismatched types -- `mut` cannot change a variable's type.
pub fn shadowing() -> usize {
    todo!("Exercise 1")
}

/// Shadowing a mutable variable with an immutable one is allowed: the new
/// binding decides. This is a common way to "freeze" a value after setup.
pub fn freeze() -> Vec<i32> {
    todo!("Exercise 1")
}
