//! Exercise 3: borrow checker case studies.
//!
//! Each case is a snippet the borrow checker rejects, with its error code
//! (provided in `CASES`). You:
//!
//! 1. write `error_codes`, which runs rustc on a snippet with
//!    `--error-format=json` and collects the codes of the errors -- the
//!    same machine-readable diagnostics cargo, rust-analyzer and CI tools use;
//! 2. write a fixed version of each snippet (`fixed`), which must compile and
//!    keep `pub fn f` with the same signature (except where the signature
//!    *is* the bug: E0106, E0515);
//! 3. write the real functions below, each the fixed form of a classic error.

use std::io;
use std::thread::JoinHandle;

use crate::toolchain;

/// A snippet the borrow checker rejects, and the code it reports.
#[derive(Debug, Clone, Copy)]
pub struct Case {
    pub code: &'static str,
    pub title: &'static str,
    pub broken: &'static str,
}

pub const CASES: &[Case] = &[
    Case {
        code: "E0382",
        title: "use after move",
        broken: r#"pub fn f() -> String { let s = String::from("a"); let t = s; format!("{s} {t}") }"#,
    },
    Case {
        code: "E0499",
        title: "two mutable borrows",
        broken: "pub fn f(v: &mut Vec<i32>) { let a = &mut v[0]; let b = &mut v[1]; *a += *b; }",
    },
    Case {
        code: "E0502",
        title: "mutating while borrowed",
        broken: "pub fn f(v: &mut Vec<i32>) { for x in v.iter() { if *x == 0 { v.push(1); } } }",
    },
    Case {
        code: "E0505",
        title: "move out while borrowed",
        broken: r#"pub fn f() -> String { let s = String::from("a"); let r = &s; drop(s); r.clone() }"#,
    },
    Case {
        code: "E0506",
        title: "assign while borrowed",
        broken: "pub fn f() -> i32 { let mut x = 1; let r = &x; x = 2; *r + x }",
    },
    Case {
        code: "E0515",
        title: "return a reference to a local",
        broken: r#"pub fn f() -> &'static str { let s = String::from("a"); &s }"#,
    },
    Case {
        code: "E0597",
        title: "borrowed value does not live long enough",
        broken: r#"pub fn f() -> usize { let r; { let s = String::from("a"); r = &s; } r.len() }"#,
    },
    Case {
        code: "E0106",
        title: "missing lifetime specifier",
        broken: "pub fn f(a: &str, b: &str) -> &str { if a.len() > b.len() { a } else { b } }",
    },
    Case {
        code: "E0373",
        title: "closure may outlive the borrowed value",
        broken: "pub fn f() -> std::thread::JoinHandle<()> { let v = vec![1]; std::thread::spawn(|| println!(\"{v:?}\")) }",
    },
];

/// The error codes rustc reports for a library snippet, in order (warnings
/// and errors without a code are ignored). Empty means it compiles.
pub fn error_codes(source: &str) -> io::Result<Vec<String>> {
    todo!("Exercise 3")
}

/// Your fix for the case with this code.
pub fn fixed(code: &str) -> Option<&'static str> {
    todo!("Exercise 3")
}

/// E0106: the longer of two strings (the first on a tie).
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    todo!("Exercise 3")
}

/// E0499: `v[i] += v[j]` for `i != j` with two simultaneous mutable
/// borrows of the slice. Returns false (and does nothing) if `i == j` or
/// either is out of range.
pub fn add_into(v: &mut [i32], i: usize, j: usize) -> bool {
    todo!("Exercise 3")
}

/// E0502: append a `1` for every `0` already in the vector.
// it must be a Vec (the function pushes); clippy only sees the todo!() body
#[allow(clippy::ptr_arg)]
pub fn push_after_zeros(v: &mut Vec<i32>) {
    todo!("Exercise 3")
}

/// E0373: sum the numbers on another thread.
pub fn spawn_sum(numbers: Vec<i64>) -> JoinHandle<i64> {
    todo!("Exercise 3")
}

/// E0597/E0515: the words of `text` as owned strings, so they outlive it.
pub fn owned_words(text: &str) -> Vec<String> {
    todo!("Exercise 3")
}
