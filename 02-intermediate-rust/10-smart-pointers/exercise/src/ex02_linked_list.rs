//! Exercise 2: A singly linked stack built from `Option<Box<Node<T>>>`.

pub struct Stack<T> {
    head: Link<T>,
    len: usize,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        todo!("Exercise 2")
    }

    pub fn push(&mut self, value: T) {
        todo!("Exercise 2")
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!("Exercise 2")
    }

    pub fn peek(&self) -> Option<&T> {
        todo!("Exercise 2")
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        todo!("Exercise 2")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 2")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 2")
    }

    /// Reverses in place by re-pointing every `next`. No allocation, no clone.
    pub fn reverse(&mut self) {
        todo!("Exercise 2")
    }

    pub fn iter(&self) -> Iter<'_, T> {
        todo!("Exercise 2")
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        todo!("Exercise 2")
    }
}

/// Without this, dropping `head` drops its Box, which drops `next`, which
/// drops *its* Box... one stack frame per node. A million nodes overflow the
/// stack. The loop below unlinks each node before it is dropped, so each drop
/// is shallow.
impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        // TODO Exercise 2: a todo!() here could abort the test run, so this is empty.
    }
}

/// Borrowing iterator: holds a reference to the next node to visit.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        todo!("Exercise 2")
    }
}

/// Consuming iterator: just pops.
pub struct IntoIter<T>(Stack<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        todo!("Exercise 2")
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        todo!("Exercise 2")
    }
}

impl<T> FromIterator<T> for Stack<T> {
    /// Pushes in order, so the *last* item ends up on top.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        todo!("Exercise 2")
    }
}

pub fn run() {
    todo!("Exercise 2")
}
