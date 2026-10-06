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
    debug_assert!(align.is_power_of_two());
    n.checked_add(align - 1).map(|v| v & !(align - 1))
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
        let buffer = (0..capacity)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect();
        Bump {
            buffer,
            offset: Cell::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn used(&self) -> usize {
        self.offset.get()
    }

    pub fn remaining(&self) -> usize {
        self.capacity() - self.used()
    }

    fn base(&self) -> *mut u8 {
        self.buffer.as_ptr() as *mut u8
    }

    /// Raw memory for `layout`, or `None` if it doesn't fit. Alignment is
    /// computed on the *address*, not the offset, since the buffer itself
    /// may sit at any address.
    pub fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        let base = self.base() as usize;
        let start = align_up(base + self.offset.get(), layout.align())? - base;
        let end = start.checked_add(layout.size())?;
        if end > self.capacity() {
            return None;
        }
        self.offset.set(end);
        // SAFETY: `start <= end <= capacity`, so the pointer is inside the buffer.
        NonNull::new(unsafe { self.base().add(start) })
    }

    /// Move `value` into the arena; the reference lives as long as the
    /// arena borrow. `T: Copy` because the arena never runs destructors.
    pub fn alloc_value<T: Copy>(&self, value: T) -> Option<&mut T> {
        let ptr = self.alloc(Layout::new::<T>())?.cast::<T>();
        // SAFETY: `ptr` is aligned for T, points to size_of::<T>() bytes no
        // other reference covers (the offset only moves forward), and lives
        // as long as `&self` (reset needs `&mut self`).
        unsafe {
            ptr.as_ptr().write(value);
            Some(&mut *ptr.as_ptr())
        }
    }

    /// Copy a slice into the arena.
    pub fn alloc_slice<T: Copy>(&self, values: &[T]) -> Option<&mut [T]> {
        let layout = Layout::array::<T>(values.len()).ok()?;
        let ptr = self.alloc(layout)?.cast::<T>();
        // SAFETY: as in `alloc_value`, for `values.len()` elements.
        unsafe {
            std::ptr::copy_nonoverlapping(values.as_ptr(), ptr.as_ptr(), values.len());
            Some(std::slice::from_raw_parts_mut(ptr.as_ptr(), values.len()))
        }
    }

    pub fn alloc_str(&self, s: &str) -> Option<&mut str> {
        let bytes = self.alloc_slice(s.as_bytes())?;
        // SAFETY: the bytes were copied from a valid `str`.
        Some(unsafe { std::str::from_utf8_unchecked_mut(bytes) })
    }

    /// Free everything. `&mut self` proves no allocation is still borrowed.
    pub fn reset(&mut self) {
        self.offset.set(0);
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
        self.offset.load(Ordering::Relaxed)
    }
}

// SAFETY: `alloc` returns null or a pointer to `layout.size()` bytes aligned
// to `layout.align()` that no other allocation overlaps (see `Sync`).
unsafe impl<const N: usize> GlobalAlloc for GlobalBump<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = self.buffer.get() as usize;
        let mut current = self.offset.load(Ordering::Relaxed);
        loop {
            let Some(start) = align_up(base + current, layout.align()).map(|a| a - base) else {
                return std::ptr::null_mut();
            };
            let Some(end) = start.checked_add(layout.size()).filter(|e| *e <= N) else {
                return std::ptr::null_mut();
            };
            match self.offset.compare_exchange_weak(
                current,
                end,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                // SAFETY: `start < end <= N`: inside the buffer.
                Ok(_) => return unsafe { (self.buffer.get() as *mut u8).add(start) },
                Err(actual) => current = actual,
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
