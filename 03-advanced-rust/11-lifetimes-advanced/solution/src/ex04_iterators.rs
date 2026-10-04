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
        Pairs { slice, index: 0 }
    }
}

impl<'a, T> Iterator for Pairs<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let pair = (self.slice.get(self.index)?, self.slice.get(self.index + 1)?);
        self.index += 1;
        Some(pair)
    }
}

/// Task 2: words, by hand. `rest` shrinks from the front as words are taken.
#[derive(Debug, Clone)]
pub struct Words<'a> {
    rest: &'a str,
}

impl<'a> Words<'a> {
    pub fn new(text: &'a str) -> Self {
        Words { rest: text }
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let trimmed = self.rest.trim_start();
        if trimmed.is_empty() {
            self.rest = trimmed;
            return None;
        }
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let (word, rest) = trimmed.split_at(end);
        self.rest = rest;
        Some(word)
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
        EveryOtherMut { slice }
    }
}

impl<'a, T> Iterator for EveryOtherMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let slice = std::mem::take(&mut self.slice);
        let (first, rest) = slice.split_first_mut()?;
        // Skip one element, if there is one, for next time. Written as one
        // expression on purpose: a `match rest.split_first_mut() { Some(..) =>
        // after, None => rest }` is rejected -- it's the same borrow-checker
        // limitation as Exercise 7c (a borrow returned on one branch is
        // assumed to last on the other branch too).
        let skip = rest.len().min(1);
        self.slice = &mut rest[skip..];
        Some(first)
    }
}

pub fn run() {
    let v = [1, 2, 3, 4];
    println!("pairs: {:?}", Pairs::new(&v).collect::<Vec<_>>());
    let increasing = Pairs::new(&[1, 5, 3, 7]).filter(|(a, b)| a < b).count();
    println!("increasing steps in [1, 5, 3, 7]: {increasing}");

    let text = String::from("  zero-copy   words\tby hand  ");
    let words: Vec<&str> = Words::new(&text).collect();
    println!("words: {words:?}");

    let mut nums = vec![1, 2, 3, 4, 5];
    for x in EveryOtherMut::new(&mut nums) {
        *x *= 10;
    }
    println!("every other x10: {nums:?}");
}
