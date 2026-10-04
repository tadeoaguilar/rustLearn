use crate::sut::*;
use std::cmp::Ordering;

/// Runs an interactive function with scripted input and returns everything it printed.
fn transcript<F>(input: &str, f: F) -> String
where
    F: FnOnce(&[u8], &mut Vec<u8>),
{
    let mut out = Vec::new();
    f(input.as_bytes(), &mut out);
    String::from_utf8(out).expect("output should be valid UTF-8")
}

// ---- Exercise 1 -------------------------------------------------------------

#[test]
fn ex1_greeting_includes_the_name() {
    assert_eq!(
        ex01_hello::greeting("John"),
        "Hello, John! Welcome to Rust!"
    );
}

#[test]
fn ex1_run_trims_the_newline_from_the_name() {
    let out = transcript("John\n", |i, o| ex01_hello::run(i, o).unwrap());
    assert!(out.starts_with("What's your name? "), "got: {out}");
    // If the '\n' is not trimmed the greeting is split across two lines.
    assert!(out.contains("Hello, John! Welcome to Rust!"), "got: {out}");
    assert!(out.contains("You're using Rust version: "), "got: {out}");
}

#[test]
fn ex1_rust_version_looks_like_a_version() {
    let v = ex01_hello::rust_version();
    assert!(
        v.split('.').count() >= 2,
        "expected something like 1.98.1, got {v}"
    );
}

// ---- Exercise 2 -------------------------------------------------------------

#[test]
fn ex2_add() {
    assert_eq!(ex02_cargo_basics::add(2, 3), 5);
    assert_eq!(ex02_cargo_basics::add(-4, 4), 0);
}

// ---- Exercise 3 -------------------------------------------------------------

#[test]
fn ex3_all_five_ways_produce_hello() {
    for s in ex03_docs_explorer::strings_five_ways() {
        assert_eq!(s, "hello");
    }
}

#[test]
fn ex3_shout_accepts_str_and_string() {
    let owned = String::from("abc");
    assert_eq!(ex03_docs_explorer::shout(&owned), "ABC");
    assert_eq!(ex03_docs_explorer::shout("xyz"), "XYZ");
}

// ---- Exercise 4 -------------------------------------------------------------

#[test]
fn ex4_check_guess() {
    use ex04_guessing_game::check_guess;
    assert_eq!(check_guess(10, 50), Ordering::Less);
    assert_eq!(check_guess(90, 50), Ordering::Greater);
    assert_eq!(check_guess(50, 50), Ordering::Equal);
}

#[test]
fn ex4_secret_is_always_in_range() {
    use ex04_guessing_game::{MAX, MIN, secret_number};
    for _ in 0..1_000 {
        let n = secret_number();
        assert!((MIN..=MAX).contains(&n), "{n} out of range");
    }
}

#[test]
fn ex4_play_reproduces_the_expected_output() {
    let mut result = None;
    let out = transcript("50\n75\n62\n", |i, o| {
        result = ex04_guessing_game::play(62, i, o).unwrap();
    });
    assert_eq!(result, Some(3));
    assert!(out.contains("Too small!"), "got: {out}");
    assert!(out.contains("Too big!"), "got: {out}");
    assert!(out.contains("You got it in 3 guesses!"), "got: {out}");
}

#[test]
fn ex4_invalid_input_is_not_counted() {
    let mut result = None;
    transcript("abc\n500\n\n7\n", |i, o| {
        result = ex04_guessing_game::play(7, i, o).unwrap();
    });
    assert_eq!(result, Some(1), "only the '7' is a valid guess");
}

#[test]
fn ex4_play_stops_when_input_runs_out() {
    let mut result = Some(0);
    transcript("1\n2\n", |i, o| {
        result = ex04_guessing_game::play(99, i, o).unwrap();
    });
    assert_eq!(
        result, None,
        "end of input must end the loop, not spin forever"
    );
}

// ---- Exercise 5 -------------------------------------------------------------

#[test]
fn ex5_commands() {
    use ex05_project_setup::{Tool, command_for};
    assert_eq!(command_for(Tool::Fmt), "cargo fmt");
    assert_eq!(command_for(Tool::Clippy), "cargo clippy");
    assert_eq!(command_for(Tool::Test), "cargo test");
}

// ---- Exercise 6 -------------------------------------------------------------

#[test]
fn ex6_fixed_sum_is_15() {
    assert_eq!(ex06_debugging::fixed_sum(), Ok(15));
}

#[test]
fn ex6_double_and_sum_of_squares() {
    assert_eq!(ex06_debugging::double(21), 42);
    assert_eq!(ex06_debugging::sum_of_squares(&[1, 2, 3]), 14);
}

// ---- Bonus ------------------------------------------------------------------

#[test]
fn bonus_build_time_is_set_at_compile_time() {
    // Anything after 2020-01-01 proves it came from the build script.
    assert!(bonus_build_script::build_time() > 1_577_836_800);
}
