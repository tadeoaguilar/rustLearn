//! Exercise 3: String vs &str.
//!
//! The returned `&str`s borrow from the input: they are views into the same
//! bytes, no copying. The compiler ties their lifetime to `s` automatically
//! (lifetime elision), so the caller can't drop the sentence and keep the word.

/// Everything up to the first space, or the whole string.
pub fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
    // Shorter: s.split(' ').next().unwrap_or("")
}

/// Everything after the last space, or the whole string.
pub fn last_word(s: &str) -> &str {
    match s.rfind(' ') {
        Some(i) => &s[i + 1..],
        None => s,
    }
}

/// Reverses by `char`, not by byte: reversing the *bytes* of "héllo" would
/// split the two-byte 'é' and produce invalid UTF-8.
///
/// (Even char-reversal breaks combining characters and emoji sequences like
/// flags. Getting that right needs the `unicode-segmentation` crate.)
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

/// Task 2: four ways to concatenate "Hello" and "World" into "Hello, World!".
pub fn concat_four_ways() -> [String; 4] {
    let hello = String::from("Hello");
    let world = String::from("World");

    // 1. `+` takes ownership of the left side and borrows the right:
    //    fn add(self, s: &str) -> String. So `hello` is moved -- clone it here
    //    because we need it again below.
    let plus = hello.clone() + ", " + &world + "!";

    // 2. format! borrows everything and allocates a new String.
    let formatted = format!("{hello}, {world}!");

    // 3. push_str appends in place to a mutable String.
    let mut pushed = String::new();
    pushed.push_str(&hello);
    pushed.push_str(", ");
    pushed.push_str(&world);
    pushed.push('!');

    // 4. String::from, then `+=` (AddAssign) -- push_str with operator syntax.
    let mut built = String::from(hello.as_str());
    built += ", ";
    built += &world;
    built += "!";

    [plus, formatted, pushed, built]
}

pub fn run() {
    let sentence = String::from("Hello Rust World");
    println!("first_word = {:?}", first_word(&sentence));
    println!("last_word  = {:?}", last_word(&sentence));
    println!("reverse    = {:?}", reverse_string("hello"));
    println!(
        "reverse    = {:?} (multi-byte chars are safe)",
        reverse_string("héllo 🦀")
    );
    for (i, s) in concat_four_ways().iter().enumerate() {
        println!("concat {}: {s}", i + 1);
    }
}
