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
    let compiled = toolchain::rustc(
        source,
        &["--crate-type=lib", "--emit=metadata", "--error-format=json"],
    )?;
    let mut codes = Vec::new();
    // one JSON object per line
    for line in compiled.stderr.lines() {
        let Ok(diagnostic) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if diagnostic["level"] != "error" {
            continue;
        }
        if let Some(code) = diagnostic["code"]["code"].as_str() {
            codes.push(code.to_string());
        }
    }
    Ok(codes)
}

/// Your fix for the case with this code.
pub fn fixed(code: &str) -> Option<&'static str> {
    Some(match code {
        // clone before moving -- or don't move: `let t = &s;`
        "E0382" => {
            r#"pub fn f() -> String { let s = String::from("a"); let t = s.clone(); format!("{s} {t}") }"#
        }
        // split the slice into two disjoint mutable halves
        "E0499" => {
            "pub fn f(v: &mut Vec<i32>) { let (left, right) = v.split_at_mut(1); left[0] += right[0]; }"
        }
        // finish reading before writing
        "E0502" => {
            "pub fn f(v: &mut Vec<i32>) { let zeros = v.iter().filter(|x| **x == 0).count(); v.extend(std::iter::repeat_n(1, zeros)); }"
        }
        // use the borrow before the move
        "E0505" => {
            r#"pub fn f() -> String { let s = String::from("a"); let r = &s; let out = r.clone(); drop(s); out }"#
        }
        // copy the value out instead of keeping a reference
        "E0506" => "pub fn f() -> i32 { let mut x = 1; let r = x; x = 2; r + x }",
        // return an owned value
        "E0515" => r#"pub fn f() -> String { let s = String::from("a"); s }"#,
        // keep the value alive as long as the reference
        "E0597" => r#"pub fn f() -> usize { let s = String::from("a"); let r = &s; r.len() }"#,
        // say the result borrows from both inputs
        "E0106" => {
            "pub fn f<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() > b.len() { a } else { b } }"
        }
        // move the vector into the thread
        "E0373" => {
            "pub fn f() -> std::thread::JoinHandle<()> { let v = vec![1]; std::thread::spawn(move || println!(\"{v:?}\")) }"
        }
        _ => return None,
    })
}

/// E0106: the longer of two strings (the first on a tie).
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if b.len() > a.len() { b } else { a }
}

/// E0499: `v[i] += v[j]` for `i != j` with two simultaneous mutable
/// borrows of the slice. Returns false (and does nothing) if `i == j` or
/// either is out of range.
pub fn add_into(v: &mut [i32], i: usize, j: usize) -> bool {
    if i == j || i >= v.len() || j >= v.len() {
        return false;
    }
    let (low, high) = v.split_at_mut(i.max(j));
    let (target, source) = if i < j {
        (&mut low[i], &high[0])
    } else {
        (&mut high[0], &low[j])
    };
    *target += *source;
    true
}

/// E0502: append a `1` for every `0` already in the vector.
pub fn push_after_zeros(v: &mut Vec<i32>) {
    let zeros = v.iter().filter(|x| **x == 0).count();
    v.extend(std::iter::repeat_n(1, zeros));
}

/// E0373: sum the numbers on another thread.
pub fn spawn_sum(numbers: Vec<i64>) -> JoinHandle<i64> {
    std::thread::spawn(move || numbers.iter().sum())
}

/// E0597/E0515: the words of `text` as owned strings, so they outlive it.
pub fn owned_words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}
