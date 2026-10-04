//! Exercise 1: Annotations and Elision.
//!
//! | Signature as written                         | Compiles? | Why |
//! |----------------------------------------------|-----------|-----|
//! | `fn first_word(s: &str) -> &str`             | yes | rule 2: one input lifetime -> the output |
//! | `fn longest(x: &str, y: &str) -> &str`       | no  | two inputs: which does the output borrow? |
//! | `fn pick_first(x: &str, y: &str) -> &str`    | no  | same -- even though the body only returns x |
//! | `fn longest_with_announcement<T>(..) -> &str`| no  | same |
//! | `fn make_greeting(name: &str) -> String`     | yes | the output isn't a reference |
//!
//! The compiler never looks at the *body* to decide a signature's lifetimes.
//! The signature is the contract; the body must honour it.

use std::fmt::Display;

/// Elided: `fn first_word<'a>(s: &'a str) -> &'a str`.
pub fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

/// "The result lives as long as the *shorter-lived* of x and y."
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

/// Task 2: `y` gets no lifetime name -- it's unrelated to the result. Callers
/// can drop `y`'s owner and keep the result, which `longest` wouldn't allow.
pub fn pick_first<'a>(x: &'a str, _y: &str) -> &'a str {
    x
}

/// Lifetime parameters and type parameters share one `<...>` list,
/// lifetimes first.
pub fn longest_with_announcement<'a, T: Display>(x: &'a str, y: &'a str, ann: T) -> &'a str {
    println!("Announcement! {ann}");
    longest(x, y)
}

pub fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

/// Task 2's caller, which compiles with `pick_first` and wouldn't with
/// `longest`:
///
/// ```text
/// result = longest(&x, &y);
///                      ^^ error[E0597]: `y` does not live long enough
/// ```
pub fn task2() -> String {
    let x = String::from("kept");
    let result;
    {
        let y = String::from("temporary");
        result = pick_first(&x, &y);
    }
    result.to_string()
}

pub fn run() {
    println!("first_word = {:?}", first_word("hello world"));
    println!("longest = {:?}", longest("short", "longer"));
    println!("pick_first after y was dropped = {:?}", task2());
    println!("{:?}", longest_with_announcement("ab", "abc", "comparing!"));
    println!("{}", make_greeting("Ferris"));
}
