//! Exercise 3: Documentation Explorer.  -- see exercises.md
//!
//! Run `cargo doc --open` and `rustup doc --std` to browse the docs.

/// Return the text "hello" built in five *different* ways
/// (String::from, to_string, ... look at the String docs).
pub fn strings_five_ways() -> [String; 5] {
    todo!("Exercise 3: five ways to create a String")
}

/// Return `text` in upper case. Note the parameter type: &str, not String.
pub fn shout(text: &str) -> String {
    todo!("Exercise 3")
}
