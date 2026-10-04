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
        let end = novel
            .find(['.', '!', '?'])
            .map(|i| i + 1)
            .unwrap_or(novel.len());
        Excerpt {
            text: &novel[..end],
        }
    }

    /// `&'a str`, NOT `&str`. Elided, `&str` would mean "borrowed from
    /// `&self`" -- from this small Excerpt struct -- and callers couldn't keep
    /// the result after the Excerpt is gone, even though the bytes live in
    /// the novel.
    pub fn text(&self) -> &'a str {
        self.text
    }

    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    /// Longest word, punctuation stripped. First one wins on ties.
    pub fn longest_word(&self) -> &'a str {
        self.text
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .fold("", |best, w| if w.len() > best.len() { w } else { best })
    }
}

/// Two references with two *independent* lifetimes.
///
/// With one lifetime -- `Highlighter<'a> { text: &'a str, keyword: &'a str }`
/// -- `'a` must fit inside both borrows, so it's capped by the keyword's
/// shorter life, and so are the results of `matching_words`. The test that
/// drops the keyword first then fails with "`keyword` does not live long
/// enough". Two lifetimes let the results depend on the text alone.
#[derive(Debug, Clone, Copy)]
pub struct Highlighter<'t, 'k> {
    text: &'t str,
    keyword: &'k str,
}

impl<'t, 'k> Highlighter<'t, 'k> {
    pub fn new(text: &'t str, keyword: &'k str) -> Self {
        Highlighter { text, keyword }
    }

    pub fn matching_words(&self) -> Vec<&'t str> {
        let needle = self.keyword.to_lowercase();
        self.text
            .split_whitespace()
            .filter(|w| w.to_lowercase().contains(&needle))
            .collect()
    }
}

pub fn run() {
    let novel = String::from("Call me Ishmael. Some years ago - never mind how long precisely...");
    let word;
    {
        let excerpt = Excerpt::first_sentence(&novel);
        println!(
            "excerpt: {:?} ({} words)",
            excerpt.text(),
            excerpt.word_count()
        );
        word = excerpt.longest_word();
    } // excerpt dropped here; `word` borrows from `novel`, so it's fine
    println!("longest word: {word}");

    let text = String::from("Rustaceans love rust and trust it");
    let found;
    {
        let keyword = String::from("RUST");
        found = Highlighter::new(&text, &keyword).matching_words();
    }
    println!("matches: {found:?}");
}
