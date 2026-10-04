//! Bonus Challenge: StrSplit with two lifetimes.
//!
//! The first attempt, with one lifetime for everything:
//!
//! ```text
//! pub struct StrSplit<'a> { remainder: Option<&'a str>, delimiter: &'a str }
//!
//! pub fn until_char(s: &str, c: char) -> &str {
//!     let delim = format!("{c}");
//!     StrSplit::new(s, &delim).next().expect("...")
//!     ^^^^^^^^^^^^^^^^^^^^^^^^ error[E0515]: cannot return value referencing local variable `delim`
//! }
//! ```
//!
//! One `'a` for both fields means `'a` is at most as long as the *shorter*
//! borrow -- the local `delim`. So the items (`&'a str`) can't outlive `delim`
//! either, even though they point into `s`. The items only ever borrow from
//! the haystack; the type should say so. Making the delimiter a generic `D`
//! (or giving it a second lifetime) does exactly that.

/// Anything that can find itself in a string.
pub trait Delimiter {
    /// Byte range (start, end) of the first match in `s`.
    fn find_next(&self, s: &str) -> Option<(usize, usize)>;
}

impl Delimiter for &str {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        todo!("Bonus")
    }
}

impl Delimiter for char {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        todo!("Bonus")
    }
}

#[derive(Debug, Clone)]
pub struct StrSplit<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimiter: D,
}

impl<'haystack, D> StrSplit<'haystack, D> {
    pub fn new(haystack: &'haystack str, delimiter: D) -> Self {
        todo!("Bonus")
    }
}

impl<'haystack, D: Delimiter> Iterator for StrSplit<'haystack, D> {
    type Item = &'haystack str;

    fn next(&mut self) -> Option<&'haystack str> {
        todo!("Bonus")
    }
}

/// Compiles now: the result borrows from `s`, and `c` is just a value.
pub fn until_char(s: &str, c: char) -> &str {
    todo!("Bonus")
}

/// And with a *temporary* string delimiter -- the case that broke the
/// single-lifetime version.
pub fn until_str<'s>(s: &'s str, delim: &str) -> &'s str {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
