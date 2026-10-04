//! Exercise 2: Generating Items.  -- see exercises.md
//!
//! Write, each with #[macro_export]:
//!
//!     count!(a b c)                        3usize, at compile time
//!     newtype!(pub Meters(f64))            struct + value() + From + Display
//!     string_enum! { pub enum Color { Red, Green, Blue } }
//!                                          enum + as_str() + ALL + COUNT + FromStr + Display
//!
//! Then use them here to define what the tests expect:
//!
//!     crate::newtype!(pub Meters(f64));
//!     crate::newtype!(pub Seconds(u64));
//!     crate::string_enum! { pub enum Color { Red, Green, Blue } }

// TODO Exercise 2: your macros, then the three definitions above.

pub fn run() {
    todo!("Exercise 2")
}
