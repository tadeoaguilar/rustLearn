//! Exercise 6: A counting allocator and a bump arena.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Forwards to the system allocator and counts what goes through it.
///
/// `const fn new` so it can initialise a `static` -- a global allocator must
/// exist before main runs.
pub struct CountingAllocator {
    allocations: AtomicUsize,
    reallocations: AtomicUsize,
    deallocations: AtomicUsize,
    bytes_in_use: AtomicUsize,
}

impl CountingAllocator {
    pub const fn new() -> Self {
        // Provided: a `static` needs this at compile time, so it can't be todo!().
        CountingAllocator {
            allocations: AtomicUsize::new(0),
            reallocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_in_use: AtomicUsize::new(0),
        }
    }

    pub fn allocations(&self) -> usize {
        todo!("Exercise 6")
    }

    /// Growing a Vec or String in place goes through `realloc`, not `alloc`.
    pub fn reallocations(&self) -> usize {
        todo!("Exercise 6")
    }

    pub fn deallocations(&self) -> usize {
        todo!("Exercise 6")
    }

    pub fn bytes_in_use(&self) -> usize {
        todo!("Exercise 6")
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        todo!("Exercise 6")
    }
}

// SAFETY: every method forwards to System, which upholds GlobalAlloc's
// contract; we only add counting, which never touches the memory. The
// counters are atomics, so this is fine from any thread. (And it never
// allocates itself -- an allocator that allocates would recurse forever.)
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!("Exercise 6")
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!("Exercise 6")
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        todo!("Exercise 6")
    }
}

/// A bump arena: strings are copied into large chunks and all freed together
/// when the arena is dropped. No per-string allocation, no per-string free.
pub struct Arena {
    chunks: RefCell<Vec<Box<[u8]>>>,
    used: Cell<usize>, // bytes used in the last chunk
    chunk_size: usize,
}

impl Arena {
    pub fn new(chunk_size: usize) -> Self {
        todo!("Exercise 6")
    }

    pub fn chunk_count(&self) -> usize {
        todo!("Exercise 6")
    }

    /// Why returning `&'a str` tied to `&'a self` is sound even though we
    /// keep pushing to `chunks`:
    ///
    /// * the bytes live in a `Box<[u8]>`, a separate heap allocation that
    ///   never moves or shrinks -- pushing to the outer Vec moves the *Box
    ///   pointers*, not the chunks they point to;
    /// * we never write to bytes we've handed out (only to `used..`);
    /// * chunks are freed only when the Arena is dropped, and the borrow
    ///   checker won't let a returned `&str` outlive the Arena.
    ///
    /// With `Vec<u8>` chunks that we grew in place, a reallocation would move
    /// the bytes and leave earlier `&str`s dangling.
    pub fn alloc_str(&self, s: &str) -> &str {
        todo!("Exercise 6")
    }
}

pub fn run() {
    todo!("Exercise 6")
}
