//! Exercise 7: Real-World Example - Text Buffer.
//!
//! Notice the receiver of each method -- it documents the method's contract:
//!   `&self`      reads only           (len, content, search)
//!   `&mut self`  modifies in place    (append, prepend, clear, replace)
//!   no self      constructs a new one (new, from_str)

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextBuffer {
    content: String,
}

impl TextBuffer {
    pub fn new() -> Self {
        todo!("Exercise 7")
    }

    // Clippy suggests implementing the `FromStr` trait for a method with this
    // name; the exercise asks for an inherent `from_str`, and FromStr would
    // force a `Result` return for a conversion that can't fail.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        todo!("Exercise 7")
    }

    pub fn append(&mut self, text: &str) {
        todo!("Exercise 7")
    }

    pub fn prepend(&mut self, text: &str) {
        todo!("Exercise 7")
    }

    pub fn clear(&mut self) {
        todo!("Exercise 7")
    }

    /// Length in **bytes**, like `String::len`. "héllo".len() == 6.
    pub fn len(&self) -> usize {
        todo!("Exercise 7")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 7")
    }

    /// Returns a borrowed view. Callers can read but not modify, and the
    /// borrow checker stops them keeping it across a later `append`.
    pub fn content(&self) -> &str {
        todo!("Exercise 7")
    }

    /// Byte offsets of every (non-overlapping) occurrence of `needle`.
    pub fn search(&self, needle: &str) -> Vec<usize> {
        todo!("Exercise 7")
    }

    pub fn replace(&mut self, from: &str, to: &str) {
        todo!("Exercise 7")
    }
}

pub fn run() {
    todo!("Exercise 7")
}
