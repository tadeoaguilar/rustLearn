//! Exercise 1: a bump allocator.
//!
//! The simplest allocator there is: a big buffer and an offset. Allocating
//! rounds the offset up to the alignment, hands out the bytes, and moves the
//! offset past them. Freeing one object does nothing; *resetting* frees
//! everything at once. That's ideal for data with one shared lifetime -- a
//! frame in a game, a request in a server, a compiler pass -- and it's
//! several times faster than a general allocator.

use std::alloc::{GlobalAlloc, Layout};
use std::cell::{Cell, UnsafeCell};
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Round `n` up to a multiple of `align` (a power of two). `None` on overflow.
pub fn align_up(n: usize, align: usize) -> Option<usize> {
    todo!("Exercise 1")
}

/// An arena that hands out memory from one buffer.
pub struct Bump {
    buffer: Box<[UnsafeCell<MaybeUninit<u8>>]>,
    offset: Cell<usize>,
}

// `&self -> &mut T` is the point of an arena: allocations come from disjoint
// parts of the buffer (the offset only moves forward), so handing out
// several `&mut`s from a shared borrow is sound -- bumpalo does the same.
#[allow(clippy::mut_from_ref)]
impl Bump {
    pub fn new(capacity: usize) -> Self {
        todo!("Exercise 1")
    }

    pub fn capacity(&self) -> usize {
        todo!("Exercise 1")
    }

    pub fn used(&self) -> usize {
        todo!("Exercise 1")
    }

    pub fn remaining(&self) -> usize {
        todo!("Exercise 1")
    }

    fn base(&self) -> *mut u8 {
        todo!("Exercise 1")
    }

    /// Raw memory for `layout`, or `None` if it doesn't fit. Alignment is
    /// computed on the *address*, not the offset, since the buffer itself
    /// may sit at any address.
    pub fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        todo!("Exercise 1")
    }

    /// Move `value` into the arena; the reference lives as long as the
    /// arena borrow. `T: Copy` because the arena never runs destructors.
    pub fn alloc_value<T: Copy>(&self, value: T) -> Option<&mut T> {
        todo!("Exercise 1")
    }

    /// Copy a slice into the arena.
    pub fn alloc_slice<T: Copy>(&self, values: &[T]) -> Option<&mut [T]> {
        todo!("Exercise 1")
    }

    pub fn alloc_str(&self, s: &str) -> Option<&mut str> {
        todo!("Exercise 1")
    }

    /// Free everything. `&mut self` proves no allocation is still borrowed.
    pub fn reset(&mut self) {
        todo!("Exercise 1")
    }
}

/// A bump allocator usable as `#[global_allocator]`: a fixed buffer and an
/// atomic offset advanced with compare-and-swap, so threads can allocate
/// without a lock. `dealloc` does nothing -- memory is never reused.
pub struct GlobalBump<const N: usize> {
    buffer: UnsafeCell<[u8; N]>,
    offset: AtomicUsize,
}

// SAFETY: threads only ever receive disjoint ranges of the buffer: each
// range is claimed by a successful compare-exchange on `offset`.
unsafe impl<const N: usize> Sync for GlobalBump<N> {}

impl<const N: usize> Default for GlobalBump<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> GlobalBump<N> {
    pub const fn new() -> Self {
        GlobalBump {
            buffer: UnsafeCell::new([0; N]),
            offset: AtomicUsize::new(0),
        }
    }

    pub fn used(&self) -> usize {
        todo!("Exercise 1")
    }
}

// SAFETY: `alloc` returns null or a pointer to `layout.size()` bytes aligned
// to `layout.align()` that no other allocation overlaps (see `Sync`).
unsafe impl<const N: usize> GlobalAlloc for GlobalBump<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!("Exercise 1")
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!("Exercise 1")
    }
}
