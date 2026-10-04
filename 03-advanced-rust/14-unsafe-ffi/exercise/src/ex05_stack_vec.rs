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
        todo!("Exercise 5")
    }

    pub const fn capacity(&self) -> usize {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 5")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 5")
    }

    /// Hands the value back when full, instead of panicking or dropping it.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        todo!("Exercise 5")
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!("Exercise 5")
    }

    pub fn as_slice(&self) -> &[T] {
        todo!("Exercise 5")
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        todo!("Exercise 5")
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    fn default() -> Self {
        todo!("Exercise 5")
    }
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        // TODO Exercise 5: a todo!() here could abort the test run, so this is empty.
    }
}

/// With Deref to [T], every slice method works: iter(), len(), sort()...
impl<T, const N: usize> Deref for StackVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        todo!("Exercise 5")
    }
}

impl<T, const N: usize> DerefMut for StackVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        todo!("Exercise 5")
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for StackVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Exercise 5")
    }
}

pub fn run() {
    todo!("Exercise 5")
}
