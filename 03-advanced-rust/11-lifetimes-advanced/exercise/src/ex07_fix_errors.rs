//! Exercise 7: Fix the Lifetime Errors.  -- see exercises.md
//!
//! Each broken snippet is below as a comment. Paste it into a scratch
//! function, read the error, then implement the fixed version underneath.
//! Ask "who owns this data, and for how long?" before reaching for an
//! annotation.

use std::collections::HashMap;

// ---- 7a ---------------------------------------------------------------------
//
// fn longest_line(text: &str) -> &str {
//     let upper = text.to_uppercase();
//     upper.lines().max_by_key(|l| l.len()).unwrap_or("")
// }

/// The longest line, upper-cased.
pub fn longest_line_upper(text: &str) -> String {
    todo!("Exercise 7a")
}

/// The longest line, as a slice of `text`.
pub fn longest_line(text: &str) -> &str {
    todo!("Exercise 7a")
}

// ---- 7b ---------------------------------------------------------------------
//
// struct Config<'a> { name: &'a str }
// fn load() -> Config<'static> {
//     let raw = std::fs::read_to_string("app.conf").unwrap();
//     Config { name: raw.trim() }
// }

/// Fix 1: a config that owns its data.
#[derive(Debug, PartialEq, Eq)]
pub struct OwnedConfig {
    pub name: String,
}

pub fn load_owned(path: &std::path::Path) -> std::io::Result<OwnedConfig> {
    todo!("Exercise 7b")
}

/// Fix 2: a config that borrows from a buffer the *caller* keeps.
#[derive(Debug, PartialEq, Eq)]
pub struct Config<'a> {
    pub name: &'a str,
}

pub fn parse_config(raw: &str) -> Config<'_> {
    todo!("Exercise 7b")
}

// ---- 7c ---------------------------------------------------------------------
//
// fn get_or_default<'m>(map: &'m mut HashMap<String, String>, key: &str) -> &'m String {
//     if let Some(v) = map.get(key) {
//         return v;
//     }
//     map.insert(key.to_string(), String::new());
//     map.get(key).unwrap()
// }
//
// Write it two ways: with the entry API, and with contains_key first.

pub fn get_or_default<'m>(map: &'m mut HashMap<String, String>, key: &str) -> &'m String {
    todo!("Exercise 7c: entry API")
}

pub fn get_or_default_two_lookups<'m>(
    map: &'m mut HashMap<String, String>,
    key: &str,
) -> &'m String {
    todo!("Exercise 7c: contains_key, then index")
}

// ---- 7d ---------------------------------------------------------------------
//
// let mut p = Parser::new("a b");
// let first = p.next_token();
// let second = p.next_token();   // error[E0499]
// println!("{first} {second}");

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        todo!("Exercise 7d")
    }

    /// Returns the next whitespace-separated token ("" at the end).
    // TODO 7d: this is the broken signature from exercises.md.
    pub fn next_token(&mut self) -> &str {
        todo!("Exercise 7d")
    }
}

// ---- 7e ---------------------------------------------------------------------
//
// fn make_adder(n: &i32) -> Box<dyn Fn(i32) -> i32> {
//     Box::new(|x| x + n)
// }

/// Fix 1: keep this signature exactly; change the body.
pub fn make_adder(n: &i32) -> Box<dyn Fn(i32) -> i32> {
    todo!("Exercise 7e")
}

/// Fix 2: keep the borrow; change the signature (it's already done here --
/// explain to yourself what `+ 'a` means).
pub fn make_adder_borrowing<'a>(n: &'a i32) -> Box<dyn Fn(i32) -> i32 + 'a> {
    todo!("Exercise 7e")
}

pub fn run() {
    todo!("Exercise 7: call your fixed functions and print the results")
}
