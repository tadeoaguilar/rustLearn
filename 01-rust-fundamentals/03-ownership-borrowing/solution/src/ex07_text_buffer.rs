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
        Self {
            content: String::new(),
        }
    }

    // Clippy suggests implementing the `FromStr` trait for a method with this
    // name; the exercise asks for an inherent `from_str`, and FromStr would
    // force a `Result` return for a conversion that can't fail.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self {
            content: s.to_string(),
        }
    }

    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn prepend(&mut self, text: &str) {
        self.content.insert_str(0, text);
    }

    pub fn clear(&mut self) {
        self.content.clear(); // keeps the allocation for reuse
    }

    /// Length in **bytes**, like `String::len`. "héllo".len() == 6.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Returns a borrowed view. Callers can read but not modify, and the
    /// borrow checker stops them keeping it across a later `append`.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Byte offsets of every (non-overlapping) occurrence of `needle`.
    pub fn search(&self, needle: &str) -> Vec<usize> {
        if needle.is_empty() {
            return Vec::new();
        }
        self.content.match_indices(needle).map(|(i, _)| i).collect()
    }

    pub fn replace(&mut self, from: &str, to: &str) {
        // `str::replace` builds a new String; we swap it in.
        self.content = self.content.replace(from, to);
    }
}

pub fn run() {
    let mut buffer = TextBuffer::new();
    buffer.append("Hello ");
    buffer.append("World");
    assert_eq!(buffer.content(), "Hello World");
    assert_eq!(buffer.len(), 11);

    buffer.prepend("Say: ");
    assert_eq!(buffer.content(), "Say: Hello World");

    // exercises.md expects vec![9, 13] here; that is a typo in the exercise.
    // "Say: Hello World" has 'l' at byte offsets 7, 8 and 14.
    let positions = buffer.search("l");
    println!("{:?} -> 'l' at {positions:?}", buffer.content());
    assert_eq!(positions, vec![7, 8, 14]);

    buffer.replace("World", "Rust");
    assert_eq!(buffer.content(), "Say: Hello Rust");
    println!("after replace: {:?}", buffer.content());

    buffer.clear();
    assert!(buffer.is_empty());
    println!("after clear: is_empty = {}", buffer.is_empty());
}
