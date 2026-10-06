//! Exercise 2: pools -- a generational slab and an object pool.
//!
//! A **slab** stores objects in a `Vec` and hands out indices instead of
//! pointers; freed slots go on a free list and are reused. Indices don't
//! dangle like pointers, but a stale index can point at a *new* object in a
//! reused slot -- so each slot carries a generation, bumped on reuse, and a
//! handle is (index, generation). ECS game engines and many async runtimes
//! are built on this.
//!
//! An **object pool** recycles expensive objects (buffers, connections)
//! instead of allocating new ones: take one, use it, and it goes back when
//! the guard is dropped.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle {
    pub index: u32,
    pub generation: u32,
}

#[derive(Debug)]
enum Slot<T> {
    Occupied {
        generation: u32,
        value: T,
    },
    Free {
        generation: u32,
        next_free: Option<u32>,
    },
}

#[derive(Debug)]
pub struct Slab<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<u32>,
    len: usize,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        todo!("Exercise 2")
    }
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        todo!("Exercise 2")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 2")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 2")
    }

    /// Slots allocated, used or free.
    pub fn capacity(&self) -> usize {
        todo!("Exercise 2")
    }

    /// Store `value`, reusing a free slot if there is one.
    pub fn insert(&mut self, value: T) -> Handle {
        todo!("Exercise 2")
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        todo!("Exercise 2")
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        todo!("Exercise 2")
    }

    /// Take the value out; the slot's generation is bumped, so `handle`
    /// (and any copy of it) is dead from now on.
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        todo!("Exercise 2")
    }

    /// Live values with their handles, in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle, &T)> {
        todo!("Exercise 2");
        #[allow(unreachable_code)]
        std::iter::empty()
    }
}

/// Recycles objects made by `make`; `reset` prepares a returned one for reuse.
pub struct ObjectPool<T> {
    free: RefCell<Vec<T>>,
    make: fn() -> T,
    reset: fn(&mut T),
    created: std::cell::Cell<usize>,
}

impl<T> ObjectPool<T> {
    pub fn new(make: fn() -> T, reset: fn(&mut T)) -> Self {
        todo!("Exercise 2")
    }

    /// A pooled object: a recycled one if available, else a new one.
    pub fn take(&self) -> Pooled<'_, T> {
        todo!("Exercise 2")
    }

    /// Objects ever created (not recycled).
    pub fn created(&self) -> usize {
        todo!("Exercise 2")
    }

    /// Objects waiting to be reused.
    pub fn available(&self) -> usize {
        todo!("Exercise 2")
    }
}

/// Borrowed from an `ObjectPool`; goes back (reset) when dropped.
pub struct Pooled<'a, T> {
    pool: &'a ObjectPool<T>,
    value: Option<T>,
}

impl<T> Deref for Pooled<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        todo!("Exercise 2")
    }
}

impl<T> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        todo!("Exercise 2")
    }
}

impl<T> Drop for Pooled<'_, T> {
    fn drop(&mut self) {
        // TODO Exercise 2: a todo!() here could abort the test run, so this is empty.
    }
}
