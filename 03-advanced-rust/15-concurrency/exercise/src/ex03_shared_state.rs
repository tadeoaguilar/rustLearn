//! Exercise 3: Shared State -- Mutex, lock ordering, Condvar.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankError {
    NoSuchAccount(usize),
    SameAccount,
    InsufficientFunds { balance: i64, requested: i64 },
}

/// One Mutex per account, so transfers between *different* accounts run in
/// parallel instead of queueing behind one big lock.
pub struct Bank {
    accounts: Vec<Mutex<i64>>,
}

impl Bank {
    pub fn new(balances: &[i64]) -> Self {
        todo!("Exercise 3")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 3")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 3")
    }

    /// The naive version locks `from`, then `to`. Thread A transferring 1->2
    /// holds lock 1 and waits for 2; thread B transferring 2->1 holds 2 and
    /// waits for 1. Neither can proceed: deadlock.
    ///
    /// The fix: a global lock order. Every thread locks the *lower index
    /// first*, so no cycle of waiting threads can form.
    pub fn transfer(&self, from: usize, to: usize, amount: i64) -> Result<(), BankError> {
        todo!("Exercise 3")
    }

    /// Locks *every* account (in index order, like transfer) so the sum is a
    /// consistent snapshot -- no transfer can be half-way through.
    pub fn total(&self) -> i64 {
        todo!("Exercise 3")
    }
}

/// `threads` threads each make `transfers` pseudo-random transfers. Returns
/// the total before and after -- they must be equal.
pub fn stress_bank(accounts: usize, threads: usize, transfers: usize) -> (i64, i64) {
    todo!("Exercise 3")
}

/// A bounded queue that *blocks*: push waits while full, pop while empty.
pub struct BlockingQueue<T> {
    items: Mutex<VecDeque<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
}

impl<T> BlockingQueue<T> {
    pub fn new(capacity: usize) -> Self {
        todo!("Exercise 3")
    }

    pub fn push(&self, item: T) {
        todo!("Exercise 3")
    }

    pub fn pop(&self) -> T {
        todo!("Exercise 3")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 3")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 3")
    }
}

pub fn run() {
    todo!("Exercise 3")
}
