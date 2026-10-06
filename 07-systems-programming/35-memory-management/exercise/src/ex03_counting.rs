//! Exercise 3: measuring allocations with a counting allocator.
//!
//! Every heap allocation in a Rust program goes through the
//! `#[global_allocator]` -- `System` (malloc) by default. Wrap it, count, and
//! you have an allocation profiler: how many allocations a function makes,
//! how many bytes are live, the peak. (`dhat` and heaptrack do this with
//! stack traces.)
//!
//! Counters are kept twice: globally (atomics), and per thread, so
//! `measure` sees only the calling thread's allocations even while other
//! threads (other tests, say) allocate too. Thread-locals initialised with
//! `const` don't allocate, which matters: an allocator that allocates while
//! allocating recurses forever.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Counting<A = System> {
    inner: A,
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    live_bytes: AtomicUsize,
    peak_bytes: AtomicUsize,
}

thread_local! {
    static THREAD_ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static THREAD_BYTES: Cell<usize> = const { Cell::new(0) };
}

impl Counting<System> {
    /// Wraps the system allocator; `const` so it can initialise a static.
    pub const fn system() -> Self {
        Counting::new(System)
    }
}

impl<A> Counting<A> {
    pub const fn new(inner: A) -> Self {
        Counting {
            inner,
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            live_bytes: AtomicUsize::new(0),
            peak_bytes: AtomicUsize::new(0),
        }
    }

    /// The global totals so far.
    pub fn snapshot(&self) -> Snapshot {
        todo!("Exercise 3")
    }

    fn record_alloc(&self, size: usize) {
        // TODO Exercise 3: count. (Empty rather than todo!(): a panic inside
        // the global allocator aborts the whole process.)
        let _ = size;
    }

    fn record_dealloc(&self, size: usize) {
        // TODO Exercise 3: count. (Empty rather than todo!(): a panic inside
        // the global allocator aborts the whole process.)
        let _ = size;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Snapshot {
    pub allocations: usize,
    pub deallocations: usize,
    pub live_bytes: usize,
    pub peak_bytes: usize,
}

// SAFETY: every call is forwarded to `inner`, which upholds GlobalAlloc's
// contract; the counting only touches atomics and const thread-locals,
// neither of which allocates.
unsafe impl<A: GlobalAlloc> GlobalAlloc for Counting<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded with the caller's guarantees.
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            self.record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarded with the caller's guarantees.
        unsafe { self.inner.dealloc(ptr, layout) };
        self.record_dealloc(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: forwarded with the caller's guarantees.
        let new = unsafe { self.inner.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            // A realloc counts as a new allocation (it may well move the data).
            self.record_dealloc(layout.size());
            self.record_alloc(new_size);
        }
        new
    }
}

/// What a closure allocated on this thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measured {
    pub allocations: usize,
    pub bytes: usize,
}

/// Run `f`, counting this thread's allocations during it. Only meaningful
/// when a `Counting` allocator is the global allocator.
pub fn measure<R>(f: impl FnOnce() -> R) -> (R, Measured) {
    todo!("Exercise 3")
}

// Functions to measure: the same results, very different allocation counts.

/// Words joined with commas, growing a `String` from empty.
pub fn join_naive(words: &[&str]) -> String {
    todo!("Exercise 3")
}

/// The same, with the exact capacity reserved first: one allocation.
pub fn join_reserved(words: &[&str]) -> String {
    todo!("Exercise 3")
}

/// Count words longer than `n` without allocating at all.
pub fn count_long_words(text: &str, n: usize) -> usize {
    todo!("Exercise 3")
}
