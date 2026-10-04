//! Exercise 8: A Lock-Free (Treiber) Stack with crossbeam-epoch.
//!
//! Lock-free: some thread always makes progress -- no thread can block the
//! others by holding a lock and getting descheduled. Built from one primitive,
//! compare-and-swap: "set head to `new` only if it's still `old`".
//!
//! The hard part isn't the CAS loop, it's *freeing memory*. After a pop
//! unlinks a node, another thread that loaded `head` a moment earlier may
//! still be reading that node. Freeing it immediately = use-after-free.
//! Epoch-based reclamation defers the free until every thread that might
//! hold a reference has moved on (`epoch::pin()` marks "I might be reading").

use crossbeam::epoch::{self, Atomic, Owned};
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

pub struct LockFreeStack<T> {
    head: Atomic<Node<T>>,
}

struct Node<T> {
    // ManuallyDrop: pop moves the value out with ptr::read; the node is freed
    // later by the epoch collector, which must not drop the value again.
    value: ManuallyDrop<T>,
    next: Atomic<Node<T>>,
}

// SAFETY: values are moved in and out, never shared by reference between
// threads, so T: Send is enough for the whole stack to be Send and Sync.
unsafe impl<T: Send> Send for LockFreeStack<T> {}
unsafe impl<T: Send> Sync for LockFreeStack<T> {}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        todo!("Exercise 8")
    }

    pub fn push(&self, value: T) {
        todo!("Exercise 8")
    }

    pub fn pop(&self) -> Option<T> {
        todo!("Exercise 8")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 8")
    }
}

impl<T> Default for LockFreeStack<T> {
    fn default() -> Self {
        todo!("Exercise 8")
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        // TODO Exercise 8: a todo!() here could abort the test run, so this is empty.
    }
}

/// `threads` threads each push `per_thread` distinct values, then everyone
/// pops until empty. Returns every popped value, sorted.
pub fn stress_stack(threads: usize, per_thread: usize) -> Vec<usize> {
    todo!("Exercise 8")
}

/// The production version of the idea: crossbeam's ArrayQueue, a bounded
/// lock-free MPMC queue. Producers and consumers run *at the same time*.
pub fn mpmc_with_array_queue(producers: usize, per_producer: usize) -> Vec<usize> {
    todo!("Exercise 8")
}

pub fn run() {
    todo!("Exercise 8")
}
