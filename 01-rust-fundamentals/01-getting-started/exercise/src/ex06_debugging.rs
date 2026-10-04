//! Exercise 6: Debugging Practice.  -- see exercises.md
//!
//! The buggy program:
//!
//!     let x = 5;
//!     let y = "10";
//!     let result = x + y;
//!
//! Paste it into a function first and read the compiler error carefully.

/// Fix the program above so it returns Ok(15).
pub fn fixed_sum() -> Result<i32, std::num::ParseIntError> {
    todo!("Exercise 6: parse y")
}

/// Return n * 2. (Then add a `;` after the expression and read the error.)
pub fn double(n: i32) -> i32 {
    todo!("Exercise 6")
}

/// Return the sum of the squares. Use `dbg!` to print each square as you go.
pub fn sum_of_squares(values: &[i32]) -> i32 {
    todo!("Exercise 6: try dbg!")
}
