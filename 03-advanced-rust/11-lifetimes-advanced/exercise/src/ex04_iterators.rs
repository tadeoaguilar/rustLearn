//! Exercise 4: Iterators That Borrow.
//!
//! The key line in each impl is `type Item = &'a ...`: items borrow from the
//! *collection* (lifetime 'a), not from the iterator. That's what lets a
//! caller keep items after the iterator is gone, or `collect()` them.

/// Task 1: adjacent pairs. `[1, 2, 3]` -> `(1, 2), (2, 3)`.
#[derive(Debug, Clone)]
pub struct Pairs<'a, T> {
    slice: &'a [T],
    index: usize,
}

impl<'a, T> Pairs<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        todo!("Exercise 4")
    }
}

impl<'a, T> Iterator for Pairs<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        todo!("Exercise 4")
    }
}

/// Task 2: words, by hand. `rest` shrinks from the front as words are taken.
#[derive(Debug, Clone)]
pub struct Words<'a> {
    rest: &'a str,
}

impl<'a> Words<'a> {
    pub fn new(text: &'a str) -> Self {
        todo!("Exercise 4")
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        todo!("Exercise 4")
    }
}

/// Task 3: every other element, mutably -- the hard one.
///
/// The naive `next` returns `&mut self.slice[0]` and then shortens
/// `self.slice`. That doesn't compile: the item would be borrowed from
/// `&mut self` (short-lived), but `Item` promises `&'a mut T`. Two `&mut` to
/// the same data can't coexist, so the compiler can't let the item outlive
/// the `self.slice` it came from.
///
/// The fix: *move* the slice out of `self` with `mem::take` (leaving an empty
/// slice behind). Now we own a `&'a mut [T]` outright, split it into the
/// item and the remainder, and put the remainder back.
#[derive(Debug)]
pub struct EveryOtherMut<'a, T> {
    slice: &'a mut [T],
}

impl<'a, T> EveryOtherMut<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        todo!("Exercise 4")
    }
}

impl<'a, T> Iterator for EveryOtherMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        todo!("Exercise 4")
    }
}

pub fn run() {
    todo!("Exercise 4")
}
