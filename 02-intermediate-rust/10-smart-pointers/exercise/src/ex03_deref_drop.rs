//! Exercise 3: Deref and Drop.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// Task 1: a box that keeps its value on the stack -- the point is Deref.
#[derive(Debug)]
pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(x: T) -> MyBox<T> {
        todo!("Exercise 3")
    }
}

/// `*my_box` becomes `*(my_box.deref())`.
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        todo!("Exercise 3")
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        todo!("Exercise 3")
    }
}

pub fn hello(name: &str) -> String {
    todo!("Exercise 3")
}

/// Deref coercion: `&MyBox<String>` -> `&String` -> `&str`, inserted by the
/// compiler. Without it you'd write `hello(&(*m)[..])`.
pub fn greet_boxed(m: &MyBox<String>) -> String {
    todo!("Exercise 3")
}

/// Task 2: records its own drop in a shared log.
pub type Log = Rc<RefCell<Vec<String>>>;

pub fn new_log() -> Log {
    todo!("Exercise 3")
}

pub struct Noisy {
    pub name: String,
    log: Log,
}

impl Noisy {
    pub fn new(name: &str, log: &Log) -> Noisy {
        todo!("Exercise 3")
    }
}

impl Drop for Noisy {
    fn drop(&mut self) {
        // TODO Exercise 3: a todo!() here could abort the test run, so this is empty.
    }
}

/// Fields are dropped in declaration order, *after* the struct's own Drop
/// (if it has one -- this one doesn't).
pub struct Pair {
    pub first: Noisy,
    pub second: Noisy,
}

/// Locals are dropped in reverse declaration order.
pub fn locals_order(log: &Log) {
    todo!("Exercise 3")
} // drops c, b, a

pub fn fields_order(log: &Log) {
    todo!("Exercise 3")
} // drops first, second

pub fn early_drop(log: &Log) {
    todo!("Exercise 3")
}

fn consume(n: Noisy, log: &Log) {
    todo!("Exercise 3")
} // n dropped here, at the end of *this* function

pub fn moved_into_function(log: &Log) {
    todo!("Exercise 3")
}

/// `let _ = x` does NOT bind (and so drops a temporary immediately);
/// `let _x = x` binds and lives to the end of scope. A classic surprise.
pub fn underscore_vs_named(log: &Log) {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
