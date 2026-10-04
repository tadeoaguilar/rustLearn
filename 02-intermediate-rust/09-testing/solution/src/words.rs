//! Word counting, two ways -- for Exercise 7's benchmark.

use std::collections::HashMap;

/// Allocates a new `String` for every word it sees.
pub fn count_words_split(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }
    counts
}

/// Borrows each word from `text`: no allocation per word. The returned map
/// can't outlive `text` -- the lifetime in the signature says so.
pub fn count_words_borrowed(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_implementations_agree() {
        let text = "the cat and the hat and the bat";
        let a = count_words_split(text);
        let b = count_words_borrowed(text);
        assert_eq!(a.len(), b.len());
        for (word, n) in &b {
            assert_eq!(a.get(*word), Some(n), "count of {word:?}");
        }
        assert_eq!(b["the"], 3);
    }
}
