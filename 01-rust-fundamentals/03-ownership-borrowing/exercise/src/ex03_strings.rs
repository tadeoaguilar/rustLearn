//! Exercise 3: String vs &str.
//!
//! The returned `&str`s borrow from the input: they are views into the same
//! bytes, no copying. The compiler ties their lifetime to `s` automatically
//! (lifetime elision), so the caller can't drop the sentence and keep the word.

/// Everything up to the first space, or the whole string.
pub fn first_word(s: &str) -> &str {
    todo!("Exercise 3")
}

/// Everything after the last space, or the whole string.
pub fn last_word(s: &str) -> &str {
    todo!("Exercise 3")
}

/// Reverses by `char`, not by byte: reversing the *bytes* of "héllo" would
/// split the two-byte 'é' and produce invalid UTF-8.
///
/// (Even char-reversal breaks combining characters and emoji sequences like
/// flags. Getting that right needs the `unicode-segmentation` crate.)
pub fn reverse_string(s: &str) -> String {
    todo!("Exercise 3")
}

/// Task 2: four ways to concatenate "Hello" and "World" into "Hello, World!".
pub fn concat_four_ways() -> [String; 4] {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
