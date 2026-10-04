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
        s.find(*self).map(|start| (start, start + self.len()))
    }
}

impl Delimiter for char {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.char_indices()
            .find(|(_, c)| c == self)
            .map(|(start, c)| (start, start + c.len_utf8()))
    }
}

#[derive(Debug, Clone)]
pub struct StrSplit<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimiter: D,
}

impl<'haystack, D> StrSplit<'haystack, D> {
    pub fn new(haystack: &'haystack str, delimiter: D) -> Self {
        StrSplit {
            remainder: Some(haystack),
            delimiter,
        }
    }
}

impl<'haystack, D: Delimiter> Iterator for StrSplit<'haystack, D> {
    type Item = &'haystack str;

    fn next(&mut self) -> Option<&'haystack str> {
        // `as_mut()?` gives `&mut &'haystack str`; updating through it changes
        // self.remainder in place. Once it's None the iterator is finished.
        let remainder = self.remainder.as_mut()?;
        match self.delimiter.find_next(remainder) {
            Some((start, end)) => {
                let until = &remainder[..start];
                *remainder = &remainder[end..];
                Some(until)
            }
            None => self.remainder.take(),
        }
    }
}

/// Compiles now: the result borrows from `s`, and `c` is just a value.
pub fn until_char(s: &str, c: char) -> &str {
    StrSplit::new(s, c)
        .next()
        .expect("StrSplit always yields at least one part")
}

/// And with a *temporary* string delimiter -- the case that broke the
/// single-lifetime version.
pub fn until_str<'s>(s: &'s str, delim: &str) -> &'s str {
    let owned = delim.to_string(); // a local that dies at the end of this function
    StrSplit::new(s, owned.as_str())
        .next()
        .expect("at least one part")
}

pub fn run() {
    println!("{:?}", StrSplit::new("a b c d", " ").collect::<Vec<_>>());
    println!("{:?}", StrSplit::new("a,b,,c,", ',').collect::<Vec<_>>());
    println!(
        "until_char(\"hello world\", 'o') = {:?}",
        until_char("hello world", 'o')
    );
    println!(
        "until_str(\"key=value\", \"=\") = {:?}",
        until_str("key=value", "=")
    );
}
