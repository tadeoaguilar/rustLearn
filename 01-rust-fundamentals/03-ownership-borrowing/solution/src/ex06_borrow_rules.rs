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
        Self::default()
    }

    /// Immutable borrow of the whole cache for as long as the returned
    /// reference is alive.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
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
        let value = match self.get(from) {
            Some(v) => v.clone(), // owned copy; the borrow of self ends here
            None => return false,
        };
        self.set(to.to_string(), value);
        true
    }
}

pub fn run() {
    // Task 1: multiple readers
    let s = String::from("hello");
    let (r1, r2, r3) = (&s, &s, &s);
    println!("{r1}, {r2}, {r3}");

    // Task 2: one writer
    let mut s = String::from("hello");
    let w = &mut s;
    w.push('!');
    // let w2 = &mut s; // error[E0499]: cannot borrow `s` as mutable more than once at a time
    println!("{w}");

    // Task 3: readers, then a writer
    let mut s = String::from("hello");
    let (r1, r2) = (&s, &s);
    println!("{r1} and {r2}"); // last use of r1, r2
    let r3 = &mut s;
    r3.push_str(" world");
    println!("{r3}");

    // Task 4
    let mut cache = Cache::new();
    cache.set("name".to_string(), "Alice".to_string());
    if let Some(name) = cache.get("name") {
        println!("Name: {name}");
    }
    cache.copy_value("name", "backup");
    println!("backup: {:?}", cache.get("backup"));
}
