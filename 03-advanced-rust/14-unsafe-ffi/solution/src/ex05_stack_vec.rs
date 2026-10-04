//! Exercise 5: A safe abstraction -- `StackVec<T, N>`.
//!
//! THE invariant, which every method relies on and preserves:
//!
//! ```text
//! items[..len] are initialised; items[len..] are not.
//! ```

use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

pub struct StackVec<T, const N: usize> {
    items: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    pub fn new() -> Self {
        // `[const { MaybeUninit::uninit() }; N]` builds an array of
        // uninitialised slots without requiring T: Copy. No unsafe needed.
        StackVec {
            items: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Hands the value back when full, instead of panicking or dropping it.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        self.items[self.len].write(value); // safe: writing never reads the old slot
        self.len += 1; // slot len-1 is now initialised: invariant holds
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1; // first: the slot is now outside items[..len]
        // SAFETY: slot `len` was initialised (it was below the old len), and
        // since len no longer covers it, nothing will read or drop it again.
        // So we move the value out exactly once.
        Some(unsafe { self.items[self.len].assume_init_read() })
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: items[..len] are initialised T's, and MaybeUninit<T> has
        // the same layout as T.
        unsafe { std::slice::from_raw_parts(self.items.as_ptr().cast::<T>(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: as above; &mut self gives exclusive access.
        unsafe { std::slice::from_raw_parts_mut(self.items.as_mut_ptr().cast::<T>(), self.len) }
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        // MaybeUninit never drops its contents; we must, for exactly the
        // initialised ones.
        // SAFETY: as_mut_slice covers exactly items[..len]; each is dropped
        // once, and the StackVec is never used again afterwards.
        unsafe { std::ptr::drop_in_place(self.as_mut_slice()) }
    }
}

/// With Deref to [T], every slice method works: iter(), len(), sort()...
impl<T, const N: usize> Deref for StackVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for StackVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for StackVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub fn run() {
    let mut v: StackVec<String, 3> = StackVec::new();
    for word in ["alpha", "beta", "gamma", "delta"] {
        match v.push(word.to_string()) {
            Ok(()) => println!("pushed {word}"),
            Err(rejected) => println!("full -- got {rejected:?} back"),
        }
    }
    v.sort_by(|a, b| b.cmp(a)); // a slice method, via DerefMut
    println!("sorted descending in place: {v:?}");
    let popped = v.pop();
    println!("pop -> {popped:?}, now {v:?} (len {})", v.len());
    println!(
        "size_of::<StackVec<u64, 4>>() = {} bytes, all inline",
        size_of::<StackVec<u64, 4>>()
    );
}
