//! Exercise 2: Structs That Borrow.
//!
//! `Excerpt<'a>` reads "an Excerpt can't outlive the text it points into".

/// A slice of some longer text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    /// Up to and including the first '.', '!' or '?', or the whole text.
    pub fn first_sentence(novel: &'a str) -> Excerpt<'a> {
        todo!("Exercise 2")
    }

    /// The excerpt's text.
    // TODO: elided, `&str` borrows from `&self`. Is that what you want?
    pub fn text(&self) -> &str {
        todo!("Exercise 2")
    }

    pub fn word_count(&self) -> usize {
        todo!("Exercise 2")
    }

    /// Longest word, punctuation stripped. First one wins on ties.
    pub fn longest_word(&self) -> &str {
        todo!("Exercise 2")
    }
}

/// Two references: the text being searched and the keyword.
#[derive(Debug, Clone, Copy)]
// TODO: one lifetime for both fields. Compiles -- but see the `signatures`
// tests, which drop the keyword before using the results.
pub struct Highlighter<'a> {
    text: &'a str,
    keyword: &'a str,
}

impl<'a> Highlighter<'a> {
    pub fn new(text: &'a str, keyword: &'a str) -> Self {
        todo!("Exercise 2")
    }

    pub fn matching_words(&self) -> Vec<&'a str> {
        todo!("Exercise 2")
    }
}

pub fn run() {
    todo!("Exercise 2")
}
