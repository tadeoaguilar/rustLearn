//! Exercise 5: Interior Mutability with RefCell and Cell.
//!
//! The borrow rules still apply -- one `&mut` or many `&` -- but `RefCell`
//! checks them at *runtime*, and panics if you break them.

use std::cell::{Cell, RefCell};

pub trait Messenger {
    fn send(&self, msg: &str);
}

pub struct LimitTracker<'a, T: Messenger> {
    messenger: &'a T,
    value: usize,
    max: usize,
}

impl<'a, T: Messenger> LimitTracker<'a, T> {
    pub fn new(messenger: &'a T, max: usize) -> LimitTracker<'a, T> {
        todo!("Exercise 5")
    }

    pub fn value(&self) -> usize {
        todo!("Exercise 5")
    }

    pub fn set_value(&mut self, value: usize) {
        todo!("Exercise 5")
    }
}

/// Task 1 + 2: a test double. `send` takes `&self`, yet records messages:
/// the `RefCell` lends out a mutable borrow at runtime, the `Cell` swaps a
/// Copy value in and out without lending anything.
#[derive(Default)]
pub struct MockMessenger {
    pub sent: RefCell<Vec<String>>,
    pub calls: Cell<usize>,
}

impl MockMessenger {
    pub fn new() -> Self {
        todo!("Exercise 5")
    }
}

impl Messenger for MockMessenger {
    fn send(&self, msg: &str) {
        todo!("Exercise 5")
    }
}

/// Task 3: two simultaneous `borrow_mut`s. Compiles fine; panics at runtime
/// with "already borrowed: BorrowMutError".
pub fn double_borrow_panics(cell: &RefCell<Vec<i32>>) {
    todo!("Exercise 5")
}

/// The non-panicking version: `try_borrow_mut` returns a Result.
pub fn double_borrow_checked(cell: &RefCell<Vec<i32>>) -> Result<(), std::cell::BorrowMutError> {
    todo!("Exercise 5")
}

pub fn run() {
    todo!("Exercise 5")
}
