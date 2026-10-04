//! Bonus Challenge: Cow -- Clone on Write.
//!
//! `Cow<'a, str>` is either `Borrowed(&'a str)` or `Owned(String)`. A function
//! that *usually* returns its input unchanged can return the borrow for free
//! and allocate only when it really has to change something.

use std::borrow::Cow;

pub fn normalize_whitespace(s: &str) -> Cow<'_, str> {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
