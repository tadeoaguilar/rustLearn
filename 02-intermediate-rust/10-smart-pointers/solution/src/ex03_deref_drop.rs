//! Exercise 3: Deref and Drop.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// Task 1: a box that keeps its value on the stack -- the point is Deref.
#[derive(Debug)]
pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

/// `*my_box` becomes `*(my_box.deref())`.
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub fn hello(name: &str) -> String {
    format!("Hello, {name}!")
}

/// Deref coercion: `&MyBox<String>` -> `&String` -> `&str`, inserted by the
/// compiler. Without it you'd write `hello(&(*m)[..])`.
pub fn greet_boxed(m: &MyBox<String>) -> String {
    hello(m)
}

/// Task 2: records its own drop in a shared log.
pub type Log = Rc<RefCell<Vec<String>>>;

pub fn new_log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

pub struct Noisy {
    pub name: String,
    log: Log,
}

impl Noisy {
    pub fn new(name: &str, log: &Log) -> Noisy {
        Noisy {
            name: name.to_string(),
            log: Rc::clone(log),
        }
    }
}

impl Drop for Noisy {
    fn drop(&mut self) {
        self.log.borrow_mut().push(format!("drop {}", self.name));
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
    let _a = Noisy::new("a", log);
    let _b = Noisy::new("b", log);
    let _c = Noisy::new("c", log);
} // drops c, b, a

pub fn fields_order(log: &Log) {
    let _pair = Pair {
        first: Noisy::new("first", log),
        second: Noisy::new("second", log),
    };
} // drops first, second

pub fn early_drop(log: &Log) {
    let a = Noisy::new("a", log);
    let _b = Noisy::new("b", log);
    drop(a); // std::mem::drop: takes ownership, so `a` is dropped right now
    log.borrow_mut().push("end of function".to_string());
}

fn consume(n: Noisy, log: &Log) {
    log.borrow_mut().push(format!("inside consume({})", n.name));
} // n dropped here, at the end of *this* function

pub fn moved_into_function(log: &Log) {
    let a = Noisy::new("a", log);
    consume(a, log);
    log.borrow_mut().push("back in caller".to_string());
}

/// `let _ = x` does NOT bind (and so drops a temporary immediately);
/// `let _x = x` binds and lives to the end of scope. A classic surprise.
pub fn underscore_vs_named(log: &Log) {
    let _ = Noisy::new("unbound", log);
    let _kept = Noisy::new("kept", log);
    log.borrow_mut().push("end of function".to_string());
}

pub fn run() {
    let m = MyBox::new(String::from("Rust"));
    println!("{}", greet_boxed(&m));
    println!("len via auto-deref: {}", m.len());

    for (name, f) in [
        ("locals", locals_order as fn(&Log)),
        ("fields", fields_order),
        ("early drop", early_drop),
        ("moved into fn", moved_into_function),
        ("let _ vs let _x", underscore_vs_named),
    ] {
        let log = new_log();
        f(&log);
        println!("{name:>15}: {:?}", log.borrow());
    }
}
