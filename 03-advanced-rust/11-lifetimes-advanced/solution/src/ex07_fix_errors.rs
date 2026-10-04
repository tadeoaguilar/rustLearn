//! Exercise 7: Fix the Lifetime Errors.
//!
//! Each broken version is quoted with the error it actually produces. The
//! lesson repeated five times: the annotation is rarely the whole fix. Ask
//! *who owns this data, and for how long?* first.

use std::collections::HashMap;

// ---- 7a ---------------------------------------------------------------------
//
// fn longest_line(text: &str) -> &str {
//     let upper = text.to_uppercase();
//     upper.lines().max_by_key(|l| l.len()).unwrap_or("")
//     ^^^^^ error[E0515]: cannot return value referencing local variable `upper`
// }
//
// `upper` is a new String owned by the function. No annotation can make a
// reference to it outlive the function. Two honest fixes:

/// Fix 1: return an owned String.
pub fn longest_line_upper(text: &str) -> String {
    text.lines()
        .max_by_key(|l| l.len())
        .unwrap_or("")
        .to_uppercase()
}

/// Fix 2: separate the borrowing part from the allocating part -- return a
/// slice of the *input*, and let the caller uppercase it if they want.
pub fn longest_line(text: &str) -> &str {
    text.lines().max_by_key(|l| l.len()).unwrap_or("")
}

// ---- 7b ---------------------------------------------------------------------
//
// fn load() -> Config<'static> {
//     let raw = std::fs::read_to_string("app.conf").unwrap();
//     Config { name: raw.trim() }
//     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ error[E0515]: cannot return value referencing local variable `raw`
// }
//
// Same disease: `raw` dies at the end of `load`. Either the struct owns its
// data, or the *caller* owns the buffer and the struct borrows from it.

/// Fix 1: own the data.
#[derive(Debug, PartialEq, Eq)]
pub struct OwnedConfig {
    pub name: String,
}

pub fn load_owned(path: &std::path::Path) -> std::io::Result<OwnedConfig> {
    let raw = std::fs::read_to_string(path)?;
    Ok(OwnedConfig {
        name: raw.trim().to_string(),
    })
}

/// Fix 2: borrow from a buffer the caller keeps alive.
#[derive(Debug, PartialEq, Eq)]
pub struct Config<'a> {
    pub name: &'a str,
}

pub fn parse_config(raw: &str) -> Config<'_> {
    Config { name: raw.trim() }
}

// ---- 7c ---------------------------------------------------------------------
//
// fn get_or_default<'m>(map: &'m mut HashMap<String, String>, key: &str) -> &'m String {
//     if let Some(v) = map.get(key) {
//         return v;
//     }
//     map.insert(key.to_string(), String::new());
//     ^^^^^^^^^^ error[E0502]: cannot borrow `*map` as mutable because it is also borrowed as immutable
//     map.get(key).unwrap()
// }
//
// This code is actually correct -- if `get` returned None, nothing is
// borrowed any more. Today's borrow checker can't see that: because `v` is
// *returned* on one path, it treats the borrow as lasting for all of 'm on
// every path ("NLL problem case #3"). The next-generation checker, Polonius,
// accepts it. Until then:

/// Fix 1: the entry API -- one lookup, no conditional return of a borrow.
/// (Costs a `to_string()` even when the key exists.)
pub fn get_or_default<'m>(map: &'m mut HashMap<String, String>, key: &str) -> &'m String {
    map.entry(key.to_string()).or_default()
}

/// Fix 2: check first, borrow after -- two lookups, no allocation when present.
pub fn get_or_default_two_lookups<'m>(
    map: &'m mut HashMap<String, String>,
    key: &str,
) -> &'m String {
    if !map.contains_key(key) {
        map.insert(key.to_string(), String::new());
    }
    &map[key]
}

// ---- 7d ---------------------------------------------------------------------
//
// fn next_token(&mut self) -> &str     // elided: the token borrows from &mut self
//
// let first = p.next_token();
// let second = p.next_token();
//             ^ error[E0499]: cannot borrow `p` as mutable more than once at a time
// println!("{first} {second}");
//
// The token points into `input`, not into the Parser -- but the elided
// signature says otherwise, so `first` keeps `p` mutably borrowed. Saying
// what we mean fixes it:

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }

    /// `&'a str`: borrowed from the input, independent of `&mut self`.
    pub fn next_token(&mut self) -> &'a str {
        let rest = self.input[self.pos..].trim_start();
        let start = self.input.len() - rest.len();
        let end = rest
            .find(char::is_whitespace)
            .map(|i| start + i)
            .unwrap_or(self.input.len());
        self.pos = end;
        &self.input[start..end]
    }
}

// ---- 7e ---------------------------------------------------------------------
//
// fn make_adder(n: &i32) -> Box<dyn Fn(i32) -> i32> {
//     Box::new(|x| x + n)
//     ^^^^^^^^^^^^^^^^^^^ error: lifetime may not live long enough
//                         error[E0373]: closure may outlive the current function, but it borrows `n`
// }
//
// `Box<dyn Fn>` with no lifetime means `Box<dyn Fn + 'static>`: the closure
// may be kept forever, so it can't borrow `n`.

/// Fix 1 (usually right): copy the value into the closure with `move`.
pub fn make_adder(n: &i32) -> Box<dyn Fn(i32) -> i32> {
    let n = *n;
    Box::new(move |x| x + n)
}

/// Fix 2: admit that the closure borrows, and limit it to 'a.
pub fn make_adder_borrowing<'a>(n: &'a i32) -> Box<dyn Fn(i32) -> i32 + 'a> {
    Box::new(move |x| x + n)
}

pub fn run() {
    let text = "short\nthe longest line\nmid line";
    println!(
        "7a: {:?} / {:?}",
        longest_line(text),
        longest_line_upper(text)
    );

    let raw = String::from("  my-app  \n");
    println!("7b: {:?}", parse_config(&raw));

    let mut map = HashMap::new();
    map.insert("a".to_string(), "present".to_string());
    // One at a time: each call holds `&mut map` for as long as its result lives.
    let a = get_or_default(&mut map, "a").clone();
    let b = get_or_default_two_lookups(&mut map, "b").clone();
    println!("7c: {a:?}, {b:?}, map now has {} keys", map.len());

    let mut p = Parser::new("let x = 5");
    let first = p.next_token();
    let second = p.next_token();
    println!("7d: {first:?} {second:?} {:?}", p.next_token());

    let n = 10;
    println!("7e: {} {}", make_adder(&n)(5), make_adder_borrowing(&n)(7));
}
