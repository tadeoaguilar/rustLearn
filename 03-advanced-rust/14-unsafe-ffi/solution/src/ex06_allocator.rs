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
        CountingAllocator {
            allocations: AtomicUsize::new(0),
            reallocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_in_use: AtomicUsize::new(0),
        }
    }

    pub fn allocations(&self) -> usize {
        self.allocations.load(Ordering::Relaxed)
    }

    /// Growing a Vec or String in place goes through `realloc`, not `alloc`.
    pub fn reallocations(&self) -> usize {
        self.reallocations.load(Ordering::Relaxed)
    }

    pub fn deallocations(&self) -> usize {
        self.deallocations.load(Ordering::Relaxed)
    }

    pub fn bytes_in_use(&self) -> usize {
        self.bytes_in_use.load(Ordering::Relaxed)
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: every method forwards to System, which upholds GlobalAlloc's
// contract; we only add counting, which never touches the memory. The
// counters are atomics, so this is fine from any thread. (And it never
// allocates itself -- an allocator that allocates would recurse forever.)
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded unchanged; the caller upholds alloc's contract.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            self.allocations.fetch_add(1, Ordering::Relaxed);
            self.bytes_in_use
                .fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the caller guarantees ptr came from this allocator (i.e.
        // from System) with this layout.
        unsafe { System.dealloc(ptr, layout) };
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_in_use
            .fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: forwarded unchanged.
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            self.reallocations.fetch_add(1, Ordering::Relaxed);
            self.bytes_in_use.fetch_add(new_size, Ordering::Relaxed);
            self.bytes_in_use
                .fetch_sub(layout.size(), Ordering::Relaxed);
        }
        new
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
        Arena {
            chunks: RefCell::new(Vec::new()),
            used: Cell::new(0),
            chunk_size: chunk_size.max(1),
        }
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.borrow().len()
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
        let len = s.len();
        let mut chunks = self.chunks.borrow_mut();
        let fits = chunks
            .last()
            .is_some_and(|c| c.len() - self.used.get() >= len);
        if !fits {
            chunks.push(vec![0u8; len.max(self.chunk_size)].into_boxed_slice());
            self.used.set(0);
        }
        let chunk = chunks.last_mut().expect("a chunk exists");
        let start = self.used.get();
        chunk[start..start + len].copy_from_slice(s.as_bytes());
        self.used.set(start + len);
        let ptr = chunk[start..].as_ptr();
        // SAFETY: the bytes were copied from a valid &str, so they're UTF-8;
        // they live in a Box<[u8]> that is never moved, mutated at this range
        // again, or freed before `self` is dropped (see above).
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) }
    }
}

pub fn run() {
    // The binary installs CountingAllocator as its #[global_allocator]; see main.rs.
    let arena = Arena::new(64);
    let words: Vec<&str> = ["arena", "allocated", "strings"]
        .iter()
        .map(|w| arena.alloc_str(w))
        .collect();
    let long = arena.alloc_str(&"x".repeat(100)); // bigger than a chunk: gets its own
    println!(
        "arena words: {words:?}, long string of {} bytes, {} chunks",
        long.len(),
        arena.chunk_count()
    );
}
