//! Exercise 1: Understanding Ownership.
//!
//! The three rules:
//! 1. Each value has exactly one owner.
//! 2. Ownership can be *moved* to another variable or function.
//! 3. When the owner goes out of scope, the value is dropped (memory freed).

// ---- Task 1 ------------------------------------------------------------------
//
// let s1 = String::from("hello");
// let s2 = s1;
// println!("{}", s1);
//                ^^ error[E0382]: borrow of moved value: `s1`
//                   value borrowed here after move
//
// `let s2 = s1` copies the (pointer, length, capacity) on the stack and makes
// s1 invalid. If both stayed valid, both would free the same heap buffer when
// they went out of scope: a double free. Three ways to fix it:

/// Fix 1: `clone()` -- a deep copy. Two owners, two heap buffers.
pub fn fix_with_clone() -> (String, String) {
    let s1 = String::from("hello");
    let s2 = s1.clone();
    (s1, s2) // both valid
}

/// Fix 2: borrow instead of move. `s2` is a reference; `s1` keeps ownership.
pub fn fix_with_reference() -> String {
    let s1 = String::from("hello");
    let s2 = &s1;
    format!("{s1} {s2}")
}

/// Fix 3: restructure so `s1` is not needed after the move -- use it first.
pub fn fix_by_restructuring() -> String {
    let s1 = String::from("hello");
    let len = s1.len(); // use s1 while we still own it...
    let s2 = s1; // ...then move it
    format!("{s2} has {len} bytes")
}

// ---- Task 2 ------------------------------------------------------------------

/// Takes ownership: the String is dropped when this function returns.
pub fn takes_ownership(s: String) -> usize {
    s.len()
} // `s` is dropped here

/// Fix A: give ownership back.
pub fn takes_and_gives_back(s: String) -> String {
    s
}

/// Fix B (the idiomatic one): don't take ownership in the first place.
pub fn borrows(s: &str) -> usize {
    s.len()
}

/// Copy types are copied, not moved: the caller's value stays usable.
pub fn makes_copy(n: i32) -> i32 {
    n * 2
}

pub fn run() {
    println!("clone:        {:?}", fix_with_clone());
    println!("reference:    {}", fix_with_reference());
    println!("restructure:  {}", fix_by_restructuring());

    let s = String::from("hello");
    let s = takes_and_gives_back(s); // moved in, moved back out
    println!("got it back:  {s}");
    println!("borrowed len: {} and still have {s}", borrows(&s));
    println!("moved len:    {} (s is gone now)", takes_ownership(s));
    // println!("{s}"); // error[E0382]: borrow of moved value: `s`

    let n = 5;
    println!(
        "copy:         makes_copy({n}) = {} and n is still {n}",
        makes_copy(n)
    );
}
