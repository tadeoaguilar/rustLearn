//! Exercise 4: Dangling References.

// ---- Task 1 ------------------------------------------------------------------
//
// fn dangle() -> &String {
//                ^ error[E0106]: missing lifetime specifier
//                  help: this function's return type contains a borrowed
//                  value, but there is no value for it to be borrowed from
//     let s = String::from("hello");
//     &s
// }
//
// `s` is dropped when `dangle` returns, so `&s` would point at freed memory.
// In C that compiles and crashes later; in Rust it is rejected up front.

/// Solution 1: return the String itself -- move ownership out to the caller.
pub fn no_dangle() -> String {
    todo!("Exercise 4")
}

/// Solution 2: a string literal lives in the binary for the whole program,
/// so a reference to it can never dangle. Its type is `&'static str`.
pub fn static_str() -> &'static str {
    todo!("Exercise 4")
}

// ---- Task 2 ------------------------------------------------------------------
//
// fn longest(x: &str, y: &str) -> &str
//                                 ^ error[E0106]: missing lifetime specifier
//
// With two reference inputs the compiler can't guess which one the output
// borrows from. The annotation says: "the result lives as long as *both*
// inputs do" -- `'a` becomes the shorter of the two lifetimes.
// (Module 11 covers lifetimes properly.)

pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    todo!("Exercise 4")
}

pub fn run() {
    todo!("Exercise 4")
}
