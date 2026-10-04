//! Exercise 2: References and Borrowing.

/// Task 1. The exercise uses `&String`; `&str` is better because it also
/// accepts string literals and slices. A `&String` converts to `&str`
/// automatically (deref coercion), so existing callers still work.
pub fn calculate_length(s: &str) -> usize {
    s.len()
} // s goes out of scope, but it doesn't own anything, so nothing is dropped

/// Task 2. A mutable reference lets the function change the caller's value.
pub fn append_world(s: &mut String) {
    s.push_str(", world");
}

// ---- Task 3 ------------------------------------------------------------------
//
// let mut s = String::from("hello");
// let r1 = &s;
// let r2 = &s;
// let r3 = &mut s;
//          ^^^^^^ error[E0502]: cannot borrow `s` as mutable because it is
//                 also borrowed as immutable
// println!("{}, {}, and {}", r1, r2, r3);
//                            -- immutable borrow later used here
//
// A reference lives until its *last use* (non-lexical lifetimes). Finish with
// r1 and r2 before creating r3 and the borrows no longer overlap:

pub fn borrowing_rules_fixed() -> String {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    let readers = format!("{r1} and {r2}"); // last use of r1 and r2

    let r3 = &mut s; // fine: no immutable borrow is alive any more
    r3.push_str(" world");

    format!("{readers}; then {s}")
}

pub fn run() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("The length of '{s1}' is {len}.");

    let mut s = String::from("hello");
    append_world(&mut s);
    println!("{s}");

    println!("{}", borrowing_rules_fixed());
}
