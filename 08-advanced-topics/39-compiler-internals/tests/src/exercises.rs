//! Exercises 2-4 run the installed rustc (offline); each such test takes a
//! fraction of a second.

use crate::sut::ex01_lint::{Config, Lint, lint_source};
use crate::sut::ex02_mir::{find, parse_mir};
use crate::sut::ex03_borrowck::{self as borrowck, CASES};
use crate::sut::ex04_desugar::{count_sugar, desugar};
use crate::sut::{bonus_unused, toolchain};

// ---------------------------------------------------------------- Exercise 1

fn lints(source: &str, max_fn_lines: usize) -> Vec<(usize, usize, Lint)> {
    lint_source(source, &Config { max_fn_lines })
        .unwrap()
        .into_iter()
        .map(|f| (f.line, f.column, f.lint))
        .collect()
}

#[test]
fn ex1_expressions() {
    let source = "fn f(o: Option<u8>, r: Result<u8, ()>) -> u32 {\n    let a = o.unwrap();\n    let b = r.expect(\"ok\");\n    todo!();\n    let s = a as u32 + b as u32;\n    dbg!(s)\n}\n";
    assert_eq!(
        lints(source, 40),
        [
            (2, 15, Lint::UnwrapUsed),
            (3, 15, Lint::ExpectUsed),
            (4, 5, Lint::DebugMacro),
            (5, 15, Lint::AsCast),
            (5, 26, Lint::AsCast),
            (6, 5, Lint::DebugMacro),
        ]
    );
    // macro arguments aren't parsed: this cast is invisible to a syntax-level lint
    assert!(lints("fn f(a: u8) { println!(\"{}\", a as u32); }", 40).is_empty());
    // a method that happens to be called unwrap with an argument, or expect with none: not ours
    assert!(
        lints(
            "fn f(x: X) { x.unwrap(1); x.expect(); x.unwrap_or(2); }",
            40
        )
        .is_empty()
    );
    assert!(lint_source("fn broken( {", &Config::default()).is_err());
}

#[test]
fn ex1_items() {
    let source = r#"
pub struct Undocumented;
/// Documented.
pub struct Documented;
pub enum E { A }
pub trait T {}
struct Private;
pub fn long_one() {
    let a = 1;
    let b = 2;
    let c = a + b;
}
impl Documented {
    pub fn method(&self) {}
    /// ok
    pub fn documented(&self) {}
}
"#;
    assert_eq!(
        lints(source, 4),
        [
            (2, 12, Lint::MissingDocs),
            (5, 10, Lint::MissingDocs),
            (6, 11, Lint::MissingDocs),
            (8, 8, Lint::LongFunction),
            (8, 8, Lint::MissingDocs),
            (14, 12, Lint::MissingDocs),
        ]
    );
    let finding = &lint_source(source, &Config { max_fn_lines: 4 }).unwrap()[3];
    assert_eq!(
        finding.to_string(),
        "8:8: long_function: `long_one` is 5 lines long (max 4)"
    );
}

#[test]
fn ex1_test_modules_are_skipped() {
    let source = "/// x\npub fn ok() {}\n#[cfg(test)]\nmod tests { fn t() { None::<u8>.unwrap(); todo!() } }\nmod other { fn t() { None::<u8>.unwrap(); } }\n";
    assert_eq!(lints(source, 40), [(5, 33, Lint::UnwrapUsed)]);
}

// ---------------------------------------------------------------- Exercise 2

const MIR_SAMPLE: &str = r#"// WARNING: This output format is intended for human consumers only
fn sum_indexed(_1: &[u32]) -> u32 {
    debug v => _1;
    bb0: {
        _2 = const 0_u32;
        _4 = PtrMetadata(copy _1);
        _3 = <std::ops::Range<usize> as IntoIterator>::into_iter(move _4) -> [return: bb1, unwind continue];
    }
    bb1: {
        assert(move _13, "index out of bounds: the length is {} but the index is {}", move _12, copy _10) -> [success: bb2, unwind continue];
    }
    bb2: {
        assert(!move (_14.1: bool), "attempt to compute `{} + {}`, which would overflow", copy _2, move _11) -> [success: bb3, unwind continue];
    }
    bb3 (cleanup): {
        resume;
    }
}

fn main::{closure#0}(_1: &{closure@src/main.rs:3:13: 3:15}) -> () {
    bb0: {
        return;
    }
}
"#;

#[test]
fn ex2_parse_text() {
    let functions = parse_mir(MIR_SAMPLE);
    assert_eq!(functions.len(), 2);
    let f = &functions[0];
    assert_eq!(
        (
            f.name.as_str(),
            f.basic_blocks,
            f.bounds_checks,
            f.overflow_checks,
            f.calls
        ),
        ("sum_indexed", 4, 1, 1, 1)
    );
    assert_eq!(functions[1].name, "main::{closure#0}");
    assert_eq!(functions[1].basic_blocks, 1);
    assert!(parse_mir("").is_empty());
}

#[test]
fn ex2_real_mir() {
    let source = "pub fn first(v: &[u8]) -> u8 { v[0] }\npub fn first_checked(v: &[u8]) -> Option<u8> { v.first().copied() }\npub fn add(a: i32, b: i32) -> i32 { a + b }\npub fn wrapping(a: i32, b: i32) -> i32 { a.wrapping_add(b) }\n";
    let debug = parse_mir(&toolchain::emit_mir(source, false).unwrap());
    assert_eq!(find(&debug, "first").unwrap().bounds_checks, 1);
    assert_eq!(
        find(&debug, "first_checked").unwrap().bounds_checks,
        0,
        "no index, no check"
    );
    assert_eq!(
        find(&debug, "add").unwrap().overflow_checks,
        1,
        "debug builds check arithmetic"
    );
    assert_eq!(find(&debug, "wrapping").unwrap().overflow_checks, 0);
    let release = parse_mir(&toolchain::emit_mir(source, true).unwrap());
    assert_eq!(
        find(&release, "add").unwrap().overflow_checks,
        0,
        "-O turns overflow checks off"
    );
    assert_eq!(
        find(&release, "first").unwrap().bounds_checks,
        1,
        "bounds checks stay"
    );
}

// ---------------------------------------------------------------- Exercise 3

#[test]
fn ex3_error_codes() {
    assert_eq!(
        borrowck::error_codes("pub fn ok() -> i32 { 1 }").unwrap(),
        Vec::<String>::new()
    );
    assert_eq!(
        borrowck::error_codes("pub fn warn() { let x = 1; }").unwrap(),
        Vec::<String>::new(),
        "warnings aren't errors"
    );
    assert_eq!(
        borrowck::error_codes("pub fn ty() -> i32 { \"s\" }").unwrap(),
        ["E0308"]
    );
    for case in CASES {
        assert_eq!(
            borrowck::error_codes(case.broken).unwrap(),
            [case.code],
            "{}: {}",
            case.code,
            case.title
        );
    }
}

#[test]
fn ex3_fixes_compile() {
    for case in CASES {
        let fixed =
            borrowck::fixed(case.code).unwrap_or_else(|| panic!("no fix for {}", case.code));
        assert!(fixed.contains("pub fn f"), "{}: keep `pub fn f`", case.code);
        assert_eq!(
            borrowck::error_codes(fixed).unwrap(),
            Vec::<String>::new(),
            "{}: {fixed}",
            case.code
        );
    }
    assert_eq!(borrowck::fixed("E9999"), None);
}

#[test]
fn ex3_fixed_functions() {
    assert_eq!(borrowck::longest("ab", "abc"), "abc");
    assert_eq!(borrowck::longest("abc", "xyz"), "abc", "the first on a tie");
    let mut v = vec![1, 10, 100];
    assert!(borrowck::add_into(&mut v, 0, 2));
    assert!(borrowck::add_into(&mut v, 2, 1));
    assert_eq!(v, [101, 10, 110]);
    assert!(!borrowck::add_into(&mut v, 1, 1) && !borrowck::add_into(&mut v, 0, 3));
    let mut zeros = vec![0, 5, 0];
    borrowck::push_after_zeros(&mut zeros);
    assert_eq!(zeros, [0, 5, 0, 1, 1]);
    assert_eq!(borrowck::spawn_sum(vec![1, 2, 3, 4]).join().unwrap(), 10);
    let words = {
        let text = String::from("outlive the text");
        borrowck::owned_words(&text)
    };
    assert_eq!(words, ["outlive", "the", "text"]);
}

// ---------------------------------------------------------------- Exercise 4

const PROGRAM: &str = r#"
use std::num::ParseIntError;

fn parse_all(items: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    let mut out = Vec::new();
    for item in items {
        out.push(item.parse::<i32>()?);
    }
    Ok(out)
}

#[derive(Debug)]
struct MyError(String);
impl From<ParseIntError> for MyError {
    fn from(e: ParseIntError) -> Self { MyError(e.to_string()) }
}

fn converted(s: &str) -> Result<i32, MyError> {
    Ok(s.parse::<i32>()? * 2)
}

fn main() {
    let mut total = 0;
    'outer: for row in vec![vec![1, 2], vec![3, 4], vec![5, 6]] {
        for x in row {
            if x == 4 { continue 'outer; }
            if x == 6 { break 'outer; }
            total += x;
        }
    }
    let squares: Vec<i32> = (1..=3).map(|n| { let mut s = 0; for _ in 0..n { s += n; } s }).collect();
    println!("{:?} {:?} {} {:?} {:?} {:?}", parse_all(&["1", "22"]), parse_all(&["x"]).is_err(), total, squares, converted("21"), converted("z"));
}
"#;

#[test]
fn ex4_no_sugar_left() {
    assert_eq!(count_sugar(PROGRAM).unwrap(), (4, 2));
    let desugared = desugar(PROGRAM).unwrap();
    assert_eq!(count_sugar(&desugared).unwrap(), (0, 0));
    assert!(
        desugared.contains("IntoIterator :: into_iter") && desugared.contains("Iterator :: next"),
        "{desugared}"
    );
    assert!(
        desugared.contains("__iter0") && desugared.contains("__iter3"),
        "a counter per loop"
    );
    assert!(desugared.contains("From :: from"), "`?` converts the error");
}

#[test]
fn ex4_same_behaviour() {
    let original = toolchain::run_program(PROGRAM).unwrap();
    assert_eq!(
        original.trim(),
        r#"Ok([1, 22]) true 11 [1, 4, 9] Ok(42) Err(MyError("invalid digit found in string"))"#
    );
    let desugared = toolchain::run_program(&desugar(PROGRAM).unwrap())
        .unwrap_or_else(|e| panic!("the desugared program must compile: {e}"));
    assert_eq!(desugared, original);
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_unused_variables() {
    let source = r#"
pub fn f(used: i32, unused_param: i32, _ignored: i32) -> i32 {
    let a = used + 1;
    let (b, c) = (a, 2);
    let d = 5;
    let shown = 3;
    let closure = |x: i32, y: i32| x + b;
    for item in [1, 2] {}
    println!("{shown} {:?}", closure(1, 2));
    d
}
pub struct S;
impl S {
    pub fn m(&self, z: u8) {}
}
"#;
    let ours = bonus_unused::unused_variables(source).unwrap();
    let pairs = |names: &[&str], f: &str| {
        names
            .iter()
            .map(|n| (f.to_string(), n.to_string()))
            .collect::<Vec<_>>()
    };
    let mut expected = pairs(&["unused_param", "c", "y", "item"], "f");
    expected.extend(pairs(&["z"], "m"));
    assert_eq!(ours, expected);

    // the same set rustc reports
    let compiled = toolchain::rustc(
        source,
        &["--crate-type=lib", "--emit=metadata", "--error-format=json"],
    )
    .unwrap();
    let mut rustc: Vec<String> = compiled
        .stderr
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|d| {
            d["message"]
                .as_str()?
                .strip_prefix("unused variable: `")?
                .strip_suffix('`')
                .map(str::to_string)
        })
        .collect();
    let mut names: Vec<String> = ours.into_iter().map(|(_, v)| v).collect();
    rustc.sort();
    names.sort();
    assert_eq!(names, rustc);
}
