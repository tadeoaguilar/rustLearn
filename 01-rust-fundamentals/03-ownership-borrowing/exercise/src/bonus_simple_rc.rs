//! Bonus Challenge: Reference Counting Simulation.
//!
//! A tiny `Rc<T>`. This is the first `unsafe` code in the course, so every
//! `unsafe` block says *why* it is sound -- a habit worth keeping.
//!
//! The raw pointer also makes `SimpleRc` automatically `!Send` and `!Sync`,
//! which is correct: the count is not atomic, so two threads touching it would
//! be a data race. (`Arc` is the thread-safe version; see module 10.)

use std::ptr::NonNull;

struct RcBox<T> {
    value: T,
    ref_count: usize,
}

pub struct SimpleRc<T> {
    // NonNull instead of `*mut`: same thing, but it can never be null, which
    // documents the invariant and lets Option<SimpleRc<T>> be pointer-sized.
    data: NonNull<RcBox<T>>,
}

impl<T> SimpleRc<T> {
    pub fn new(value: T) -> Self {
        todo!("Bonus")
    }

    pub fn get(&self) -> &T {
        todo!("Bonus")
    }

    pub fn ref_count(&self) -> usize {
        todo!("Bonus")
    }
}

// The exercise defines an inherent `fn clone(&self)`. Implementing the Clone
// *trait* instead means SimpleRc works anywhere a `T: Clone` is expected.
impl<T> Clone for SimpleRc<T> {
    fn clone(&self) -> Self {
        todo!("Bonus")
    }
}

impl<T> Drop for SimpleRc<T> {
    fn drop(&mut self) {
        // TODO Bonus: a todo!() here could abort the test run, so this is empty.
    }
}

pub fn run() {
    todo!("Bonus")
}
