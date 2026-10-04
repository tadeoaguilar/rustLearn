//! Bonus Challenge: Cow -- Clone on Write.
//!
//! `Cow<'a, str>` is either `Borrowed(&'a str)` or `Owned(String)`. A function
//! that *usually* returns its input unchanged can return the borrow for free
//! and allocate only when it really has to change something.

use std::borrow::Cow;

pub fn normalize_whitespace(s: &str) -> Cow<'_, str> {
    let needs_work = s.starts_with(char::is_whitespace)
        || s.ends_with(char::is_whitespace)
        || s.contains(|c: char| c.is_whitespace() && c != ' ')
        || s.contains("  ");
    if !needs_work {
        return Cow::Borrowed(s);
    }
    Cow::Owned(s.split_whitespace().collect::<Vec<_>>().join(" "))
}

pub fn run() {
    for s in ["already fine", "too   many", "\ttabs\tand\nnewlines ", ""] {
        let out = normalize_whitespace(s);
        let kind = if matches!(out, Cow::Borrowed(_)) {
            "Borrowed (no allocation)"
        } else {
            "Owned (allocated)"
        };
        println!("{s:?} -> {out:?}  [{kind}]");
    }
}
