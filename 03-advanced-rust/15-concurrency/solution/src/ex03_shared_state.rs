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
        Bank {
            accounts: balances.iter().map(|&b| Mutex::new(b)).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    /// The naive version locks `from`, then `to`. Thread A transferring 1->2
    /// holds lock 1 and waits for 2; thread B transferring 2->1 holds 2 and
    /// waits for 1. Neither can proceed: deadlock.
    ///
    /// The fix: a global lock order. Every thread locks the *lower index
    /// first*, so no cycle of waiting threads can form.
    pub fn transfer(&self, from: usize, to: usize, amount: i64) -> Result<(), BankError> {
        if from == to {
            return Err(BankError::SameAccount);
        }
        for i in [from, to] {
            if i >= self.accounts.len() {
                return Err(BankError::NoSuchAccount(i));
            }
        }
        let (first, second) = if from < to { (from, to) } else { (to, from) };
        let mut a = self.accounts[first].lock().expect("account lock");
        let mut b = self.accounts[second].lock().expect("account lock");
        let (from_balance, to_balance) = if from < to {
            (&mut *a, &mut *b)
        } else {
            (&mut *b, &mut *a)
        };
        if *from_balance < amount {
            return Err(BankError::InsufficientFunds {
                balance: *from_balance,
                requested: amount,
            });
        }
        *from_balance -= amount;
        *to_balance += amount;
        Ok(())
    }

    /// Locks *every* account (in index order, like transfer) so the sum is a
    /// consistent snapshot -- no transfer can be half-way through.
    pub fn total(&self) -> i64 {
        let guards: Vec<_> = self
            .accounts
            .iter()
            .map(|m| m.lock().expect("account lock"))
            .collect();
        guards.iter().map(|g| **g).sum()
    }
}

/// `threads` threads each make `transfers` pseudo-random transfers. Returns
/// the total before and after -- they must be equal.
pub fn stress_bank(accounts: usize, threads: usize, transfers: usize) -> (i64, i64) {
    let bank = Arc::new(Bank::new(&vec![1_000; accounts]));
    let before = bank.total();
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let bank = Arc::clone(&bank);
            thread::spawn(move || {
                let mut seed = 0x9E37_79B9_7F4A_7C15_u64 ^ (t as u64 + 1);
                let mut next = move || {
                    // xorshift64: a tiny deterministic PRNG, good enough here
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed
                };
                for _ in 0..transfers {
                    let from = (next() % accounts as u64) as usize;
                    let to = (next() % accounts as u64) as usize;
                    let amount = (next() % 200) as i64;
                    let _ = bank.transfer(from, to, amount); // failures are fine; deadlocks aren't
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("transfer thread panicked");
    }
    (before, bank.total())
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
        assert!(capacity > 0, "capacity must be positive");
        BlockingQueue {
            items: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        }
    }

    pub fn push(&self, item: T) {
        let mut items = self.items.lock().expect("queue lock");
        // `while`, never `if`: wait() can return spuriously, or another
        // producer may have filled the slot first.
        while items.len() == self.capacity {
            items = self.not_full.wait(items).expect("queue lock");
        }
        items.push_back(item);
        self.not_empty.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut items = self.items.lock().expect("queue lock");
        // wait_while is the same loop, written for you.
        items = self
            .not_empty
            .wait_while(items, |q| q.is_empty())
            .expect("queue lock");
        let item = items.pop_front().expect("not empty");
        self.not_full.notify_one();
        item
    }

    pub fn len(&self) -> usize {
        self.items.lock().expect("queue lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn run() {
    let (before, after) = stress_bank(10, 8, 1_000);
    println!("8 threads x 1000 transfers: total {before} -> {after}");

    let q = Arc::new(BlockingQueue::new(4));
    let producer = {
        let q = Arc::clone(&q);
        thread::spawn(move || {
            for i in 0..20 {
                q.push(i); // blocks whenever 4 items are waiting
            }
        })
    };
    let received: Vec<i32> = (0..20).map(|_| q.pop()).collect();
    producer.join().unwrap();
    println!(
        "BlockingQueue(4) passed 20 items in order: {}",
        received == (0..20).collect::<Vec<_>>()
    );
}
