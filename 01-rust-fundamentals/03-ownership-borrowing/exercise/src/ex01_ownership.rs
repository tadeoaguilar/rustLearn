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
    todo!("Exercise 1")
}

/// Fix 2: borrow instead of move. `s2` is a reference; `s1` keeps ownership.
pub fn fix_with_reference() -> String {
    todo!("Exercise 1")
}

/// Fix 3: restructure so `s1` is not needed after the move -- use it first.
pub fn fix_by_restructuring() -> String {
    todo!("Exercise 1")
}

// ---- Task 2 ------------------------------------------------------------------

/// Takes ownership: the String is dropped when this function returns.
pub fn takes_ownership(s: String) -> usize {
    todo!("Exercise 1")
} // `s` is dropped here

/// Fix A: give ownership back.
pub fn takes_and_gives_back(s: String) -> String {
    todo!("Exercise 1")
}

/// Fix B (the idiomatic one): don't take ownership in the first place.
pub fn borrows(s: &str) -> usize {
    todo!("Exercise 1")
}

/// Copy types are copied, not moved: the caller's value stays usable.
pub fn makes_copy(n: i32) -> i32 {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
