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
        LimitTracker {
            messenger,
            value: 0,
            max,
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn set_value(&mut self, value: usize) {
        self.value = value;
        let percentage = self.value as f64 / self.max as f64;
        if percentage >= 1.0 {
            self.messenger.send("Error: You are over your quota!");
        } else if percentage >= 0.9 {
            self.messenger
                .send("Urgent warning: You've used up over 90% of your quota!");
        } else if percentage >= 0.75 {
            self.messenger
                .send("Warning: You've used up over 75% of your quota!");
        }
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
        Self::default()
    }
}

impl Messenger for MockMessenger {
    fn send(&self, msg: &str) {
        self.sent.borrow_mut().push(msg.to_string());
        self.calls.set(self.calls.get() + 1);
    }
}

/// Task 3: two simultaneous `borrow_mut`s. Compiles fine; panics at runtime
/// with "already borrowed: BorrowMutError".
pub fn double_borrow_panics(cell: &RefCell<Vec<i32>>) {
    let mut first = cell.borrow_mut();
    let mut second = cell.borrow_mut(); // panics here
    first.push(1);
    second.push(2);
}

/// The non-panicking version: `try_borrow_mut` returns a Result.
pub fn double_borrow_checked(cell: &RefCell<Vec<i32>>) -> Result<(), std::cell::BorrowMutError> {
    let mut first = cell.borrow_mut();
    let mut second = cell.try_borrow_mut()?; // Err while `first` is alive
    first.push(1);
    second.push(2);
    Ok(())
}

pub fn run() {
    let mock = MockMessenger::new();
    let mut tracker = LimitTracker::new(&mock, 100);
    for v in [50, 80, 95, 120] {
        tracker.set_value(v);
    }
    println!("sent {} messages:", mock.calls.get());
    for m in mock.sent.borrow().iter() {
        println!("  {m}");
    }
    let cell = RefCell::new(vec![]);
    println!(
        "double_borrow_checked -> {:?}",
        double_borrow_checked(&cell).map_err(|e| e.to_string())
    );
}
