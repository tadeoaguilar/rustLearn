//! Exercise 3: Compile-Time Validation.  -- see exercises.md
//!
//! Write, each with #[macro_export]:
//!
//!     const_assert!(cond)      `const _: () = assert!(cond);`
//!     nonzero!(n)              NonZeroU32, n == 0 is a compile error
//!     hex_color!("#rrggbb")    an Rgb, bad input is a compile error
//!
//! hex_color! should call parse_hex_color (below) inside a `const { }` block,
//! via `$crate::ex03_validation::parse_hex_color`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexError {
    MissingHash,
    WrongLength,
    BadDigit,
}

/// Parses "#rrggbb". Must be a `const fn`: loops over `s.as_bytes()`, no
/// iterators, no `?`.
pub const fn parse_hex_color(s: &str) -> Result<Rgb, HexError> {
    todo!()
}

// TODO Exercise 3: your macros here. Then:
// pub const ORANGE: Rgb = crate::hex_color!("#ff8800");

pub fn run() {
    todo!("Exercise 3")
}
