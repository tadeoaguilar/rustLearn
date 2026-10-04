//! Exercise 6: Debugging Practice.
//!
//! The buggy program from the exercise:
//!
//! ```text
//! let x = 5;
//! let y = "10";
//! let result = x + y;   // error[E0277]: cannot add `&str` to `{integer}`
//! ```
//!
//! Rust never converts between types implicitly. `"10"` is text; to add it to
//! a number you must *parse* it, and parsing can fail, so `parse` returns a
//! `Result` you have to deal with. That is the whole fix.

/// The fixed version of the exercise's program.
pub fn fixed_sum() -> Result<i32, std::num::ParseIntError> {
    let x = 5;
    let y = "10";
    let y: i32 = y.parse()?; // shadowing: the text `y` is replaced by the number `y`
    Ok(x + y)
}

/// Bug #3 from the exercise, "missing semicolon in return", is really the
/// reverse: in Rust the *last expression without a semicolon* is the return
/// value. Adding a `;` turns it into a statement, the block evaluates to `()`
/// and you get `error[E0308]: mismatched types ... expected i32, found ()`.
pub fn double(n: i32) -> i32 {
    n * 2 // <- no semicolon, this is the return value
}

/// `dbg!` prints the file, line, expression *and* value to stderr, then
/// returns the value -- so you can wrap it around any expression without
/// restructuring your code. `println!` prints only what you format, to stdout,
/// and returns `()`.
pub fn sum_of_squares(values: &[i32]) -> i32 {
    values
        .iter()
        .map(|v| dbg!(v * v)) // prints e.g. [src/ex06_debugging.rs:36:18] v * v = 9
        .sum()
}
