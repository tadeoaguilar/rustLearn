//! Exercise 6: Mutable vs Immutable Borrow.
//!
//! The rule: at any moment you can have EITHER one `&mut T` OR any number of
//! `&T` -- never both. Tasks 1-3 in exercises.md are shown in `run()`.

use std::collections::HashMap;

/// Task 4: a cache with read (`&self`) and write (`&mut self`) access.
#[derive(Debug, Default)]
pub struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    pub fn new() -> Self {
        todo!("Exercise 6")
    }

    /// Immutable borrow of the whole cache for as long as the returned
    /// reference is alive.
    pub fn get(&self, key: &str) -> Option<&String> {
        todo!("Exercise 6")
    }

    pub fn set(&mut self, key: String, value: String) {
        todo!("Exercise 6")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 6")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 6")
    }

    /// "Can we call get and set at the same time?" -- No:
    ///
    /// ```text
    /// if let Some(name) = cache.get("name") {         // immutable borrow starts
    ///     cache.set("copy".into(), name.clone());
    ///     ^^^^^ error[E0502]: cannot borrow `cache` as mutable because it is
    ///           also borrowed as immutable
    /// }                                               // ...and ends here
    /// ```
    ///
    /// `name` points *into* the HashMap. `set` might make the map grow and
    /// move its storage, leaving `name` dangling. The fix: end the immutable
    /// borrow before mutating, by cloning the value out first.
    pub fn copy_value(&mut self, from: &str, to: &str) -> bool {
        todo!("Exercise 6")
    }
}

pub fn run() {
    todo!("Exercise 6")
}
