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
        Self::new()
    }
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        Slab {
            slots: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Slots allocated, used or free.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Store `value`, reusing a free slot if there is one.
    pub fn insert(&mut self, value: T) -> Handle {
        self.len += 1;
        match self.free_head {
            Some(index) => {
                let Slot::Free {
                    generation,
                    next_free,
                } = self.slots[index as usize]
                else {
                    unreachable!("the free list only holds free slots");
                };
                self.free_head = next_free;
                self.slots[index as usize] = Slot::Occupied { generation, value };
                Handle { index, generation }
            }
            None => {
                let index = self.slots.len() as u32;
                self.slots.push(Slot::Occupied {
                    generation: 0,
                    value,
                });
                Handle {
                    index,
                    generation: 0,
                }
            }
        }
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        match self.slots.get(handle.index as usize)? {
            Slot::Occupied { generation, value } if *generation == handle.generation => Some(value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        match self.slots.get_mut(handle.index as usize)? {
            Slot::Occupied { generation, value } if *generation == handle.generation => Some(value),
            _ => None,
        }
    }

    /// Take the value out; the slot's generation is bumped, so `handle`
    /// (and any copy of it) is dead from now on.
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        self.get(handle)?;
        let freed = Slot::Free {
            generation: handle.generation.wrapping_add(1),
            next_free: self.free_head,
        };
        let Slot::Occupied { value, .. } =
            std::mem::replace(&mut self.slots[handle.index as usize], freed)
        else {
            unreachable!("checked by get");
        };
        self.free_head = Some(handle.index);
        self.len -= 1;
        Some(value)
    }

    /// Live values with their handles, in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Occupied { generation, value } => Some((
                    Handle {
                        index: i as u32,
                        generation: *generation,
                    },
                    value,
                )),
                Slot::Free { .. } => None,
            })
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
        ObjectPool {
            free: RefCell::new(Vec::new()),
            make,
            reset,
            created: std::cell::Cell::new(0),
        }
    }

    /// A pooled object: a recycled one if available, else a new one.
    pub fn take(&self) -> Pooled<'_, T> {
        let value = self.free.borrow_mut().pop().unwrap_or_else(|| {
            self.created.set(self.created.get() + 1);
            (self.make)()
        });
        Pooled {
            pool: self,
            value: Some(value),
        }
    }

    /// Objects ever created (not recycled).
    pub fn created(&self) -> usize {
        self.created.get()
    }

    /// Objects waiting to be reused.
    pub fn available(&self) -> usize {
        self.free.borrow().len()
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
        self.value.as_ref().expect("present until drop")
    }
}

impl<T> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("present until drop")
    }
}

impl<T> Drop for Pooled<'_, T> {
    fn drop(&mut self) {
        if let Some(mut value) = self.value.take() {
            (self.pool.reset)(&mut value);
            self.pool.free.borrow_mut().push(value);
        }
    }
}
