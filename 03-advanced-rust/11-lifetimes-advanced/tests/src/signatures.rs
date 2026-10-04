//! Lifetime signature checks. Every test here is about whether it
//! *compiles*: each keeps a borrowed result alive after something else it
//! could have been tied to has gone away. If your signature is too strict,
//! you get a compile error here -- read it as a failing test.
//!
//!     cargo test -p m11-lifetimes-advanced-tests --features mine,signatures

use crate::sut::*;

#[test]
fn ex1_pick_first_result_outlives_second_argument() {
    // Compile-time check: only compiles if pick_first's result is tied to x alone.
    let x = String::from("kept");
    let result;
    {
        let y = String::from("temporary");
        result = ex01_annotations::pick_first(&x, &y);
    }
    assert_eq!(result, "kept");
    assert_eq!(ex01_annotations::task2(), "kept");
}

#[test]
fn ex2_results_outlive_the_excerpt() {
    // Compile-time check: requires `-> &'a str`, not `-> &str`.
    let novel = String::from("Call me Ishmael. Some years ago...");
    let word;
    let text;
    {
        let excerpt = ex02_structs::Excerpt::first_sentence(&novel);
        word = excerpt.longest_word();
        text = excerpt.text();
    }
    assert_eq!(word, "Ishmael");
    assert_eq!(text, "Call me Ishmael.");
}

#[test]
fn ex2_highlighter_results_outlive_the_keyword() {
    // Compile-time check: requires two lifetimes on Highlighter.
    let text = String::from("Rustaceans love rust and trust it");
    let found;
    {
        let keyword = String::from("RUST");
        found = ex02_structs::Highlighter::new(&text, &keyword).matching_words();
    }
    assert_eq!(found, vec!["Rustaceans", "rust", "trust"]);
}

#[test]
fn ex3_header_values_outlive_the_request() {
    // Compile-time check: header() must return &'a str.
    let raw = String::from("GET / HTTP/1.1\r\nHost: h\r\n\r\n");
    let host;
    {
        let req = ex03_parser::parse_request(&raw).unwrap();
        host = req.header("host");
    }
    assert_eq!(host, Some("h"));
}

#[test]
fn ex7d_tokens_outlive_the_mutable_borrow() {
    // Compile-time check: two tokens alive at once requires `-> &'a str`.
    let mut p = ex07_fix_errors::Parser::new("let x = 5");
    let first = p.next_token();
    let second = p.next_token();
    assert_eq!((first, second), ("let", "x"));
    assert_eq!(p.next_token(), "=");
    assert_eq!(p.next_token(), "5");
    assert_eq!(p.next_token(), "");
}

#[test]
fn ex4_words_outlive_the_iterator() {
    let text = String::from("x y");
    let first = {
        let mut it = ex04_iterators::Words::new(&text);
        it.next().unwrap()
    };
    assert_eq!(first, "x");
}
