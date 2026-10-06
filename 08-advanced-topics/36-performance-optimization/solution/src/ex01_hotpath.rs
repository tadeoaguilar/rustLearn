//! Exercise 1: profile, then fix the hot path.
//!
//! `top_words_naive` is correct and slow -- the kind of code a profiler
//! (`cargo flamegraph`, `samply`, Instruments) points at. The profile shows
//! the time going to: allocating a lowercase `String` for every word
//! *occurrence*, hashing with SipHash (DoS-resistant, slower than needed
//! here), and sorting every distinct word when only the top few are wanted.
//! `top_words_fast` returns exactly the same answer and fixes all three.
//!
//! A word is a run of ASCII letters, digits and `'`; everything else
//! (including non-ASCII) separates words; words are compared lowercased.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// The slow version (provided: it's the baseline and the reference answer).
pub fn top_words_naive(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '\'')) {
        if !word.is_empty() {
            *counts.entry(word.to_ascii_lowercase()).or_insert(0) += 1;
        }
    }
    let mut all: Vec<(String, usize)> = counts.into_iter().collect();
    all.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    all.truncate(n);
    all
}

/// FNV-1a: a tiny, fast, non-cryptographic hash. Fine for keys an attacker
/// doesn't control; SipHash (the default) is what you want when they do.
pub struct Fnv1a(u64);

impl Default for Fnv1a {
    fn default() -> Self {
        Fnv1a(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for Fnv1a {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub type FnvBuildHasher = BuildHasherDefault<Fnv1a>;

/// The same answer as `top_words_naive`, fast:
/// - scan bytes; lowercase each word into one reused buffer
/// - look the buffer up by `&str`; allocate a `String` only for a *new* word
/// - hash with FNV-1a
/// - `select_nth_unstable_by` to find the top `n`, then sort just those
pub fn top_words_fast(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize, FnvBuildHasher> =
        HashMap::with_capacity_and_hasher(1024, FnvBuildHasher::default());
    let mut word = String::with_capacity(32);
    let mut flush = |word: &mut String| {
        if !word.is_empty() {
            match counts.get_mut(word.as_str()) {
                Some(c) => *c += 1,
                None => {
                    counts.insert(word.clone(), 1);
                }
            }
            word.clear();
        }
    };
    for &b in text.as_bytes() {
        if b.is_ascii_alphanumeric() || b == b'\'' {
            word.push(b.to_ascii_lowercase() as char);
        } else {
            flush(&mut word);
        }
    }
    flush(&mut word);

    let order =
        |a: &(String, usize), b: &(String, usize)| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0));
    let mut all: Vec<(String, usize)> = counts.into_iter().collect();
    if n == 0 {
        return Vec::new();
    }
    if n < all.len() {
        all.select_nth_unstable_by(n - 1, order);
        all.truncate(n);
    }
    all.sort_unstable_by(order);
    all
}

/// Test data: `words` pseudo-random words from a Zipf-ish vocabulary (a few
/// very common words, a long tail), deterministic.
pub fn sample_text(words: usize) -> String {
    const VOCAB: [&str; 16] = [
        "the",
        "of",
        "and",
        "Rust",
        "memory",
        "safety",
        "fast",
        "borrow",
        "checker",
        "lifetime",
        "trait",
        "async",
        "zero-cost",
        "abstraction",
        "don't",
        "fearless",
    ];
    let mut seed: u64 = 42;
    let mut out = String::with_capacity(words * 8);
    for i in 0..words {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let r = (seed >> 33) as usize;
        // squaring biases towards small indices: common words dominate
        let idx = (r % 16) * (r % 16) / 16;
        let w = if r.is_multiple_of(50) {
            format!("rare{}", r % 1000)
        } else {
            VOCAB[idx].to_string()
        };
        out.push_str(&w);
        out.push_str(if i % 12 == 11 { ".\n" } else { " " });
    }
    out
}
