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
        LockFreeStack {
            head: Atomic::null(),
        }
    }

    pub fn push(&self, value: T) {
        let mut node = Owned::new(Node {
            value: ManuallyDrop::new(value),
            next: Atomic::null(),
        });
        let guard = epoch::pin();
        loop {
            let head = self.head.load(Relaxed, &guard);
            node.next.store(head, Relaxed);
            // Release: our writes to `node` become visible to whoever Acquires `head`.
            match self
                .head
                .compare_exchange(head, node, Release, Relaxed, &guard)
            {
                Ok(_) => return,
                Err(e) => node = e.new, // someone else pushed/popped first: retry with our node
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let guard = epoch::pin();
        loop {
            let head = self.head.load(Acquire, &guard);
            // SAFETY: we're pinned, so nodes reachable from head aren't freed
            // while we look at them.
            let h = unsafe { head.as_ref() }?;
            let next = h.next.load(Relaxed, &guard);
            if self
                .head
                .compare_exchange(head, next, Relaxed, Relaxed, &guard)
                .is_ok()
            {
                // SAFETY: the CAS unlinked `head`; we're the only thread that
                // will ever take its value. defer_destroy frees the node once
                // no pinned thread can still see it.
                unsafe {
                    guard.defer_destroy(head);
                    return Some(ManuallyDrop::into_inner(ptr::read(&h.value)));
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        let guard = epoch::pin();
        self.head.load(Acquire, &guard).is_null()
    }
}

impl<T> Default for LockFreeStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// `threads` threads each push `per_thread` distinct values, then everyone
/// pops until empty. Returns every popped value, sorted.
pub fn stress_stack(threads: usize, per_thread: usize) -> Vec<usize> {
    let stack = LockFreeStack::new();
    std::thread::scope(|s| {
        for t in 0..threads {
            let stack = &stack;
            s.spawn(move || {
                for i in 0..per_thread {
                    stack.push(t * per_thread + i);
                }
            });
        }
    });
    let popped = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                let mut mine = Vec::new();
                while let Some(v) = stack.pop() {
                    mine.push(v);
                }
                popped.lock().unwrap().extend(mine);
            });
        }
    });
    let mut all = popped.into_inner().unwrap();
    all.sort_unstable();
    all
}

/// The production version of the idea: crossbeam's ArrayQueue, a bounded
/// lock-free MPMC queue. Producers and consumers run *at the same time*.
pub fn mpmc_with_array_queue(producers: usize, per_producer: usize) -> Vec<usize> {
    use crossbeam::queue::ArrayQueue;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let queue = ArrayQueue::new(64);
    let total = producers * per_producer;
    let taken = AtomicUsize::new(0);
    let out = std::sync::Mutex::new(Vec::with_capacity(total));
    std::thread::scope(|s| {
        for p in 0..producers {
            let queue = &queue;
            s.spawn(move || {
                for i in 0..per_producer {
                    let mut v = p * per_producer + i;
                    while let Err(back) = queue.push(v) {
                        v = back; // full: try again
                        std::hint::spin_loop();
                    }
                }
            });
        }
        for _ in 0..producers {
            s.spawn(|| {
                let mut mine = Vec::new();
                while taken.load(Ordering::SeqCst) < total {
                    match queue.pop() {
                        Some(v) => {
                            mine.push(v);
                            taken.fetch_add(1, Ordering::SeqCst);
                        }
                        None => std::hint::spin_loop(),
                    }
                }
                out.lock().unwrap().extend(mine);
            });
        }
    });
    let mut all = out.into_inner().unwrap();
    all.sort_unstable();
    all
}

pub fn run() {
    let s = LockFreeStack::new();
    s.push(1);
    s.push(2);
    println!("pop, pop, pop: {:?} {:?} {:?}", s.pop(), s.pop(), s.pop());
    let all = stress_stack(8, 10_000);
    println!(
        "8 threads x 10,000: popped {} values, all distinct and present: {}",
        all.len(),
        all == (0..80_000).collect::<Vec<_>>()
    );
    let all = mpmc_with_array_queue(4, 10_000);
    println!(
        "ArrayQueue MPMC 4x10,000: {} values, correct: {}",
        all.len(),
        all == (0..40_000).collect::<Vec<_>>()
    );
}
