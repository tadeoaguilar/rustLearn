//! Exercise 7: Generic Data Structures -- a Stack<T>.

#[derive(Debug, Clone, PartialEq)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        todo!("Exercise 7")
    }

    pub fn push(&mut self, item: T) {
        todo!("Exercise 7")
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!("Exercise 7")
    }

    pub fn peek(&self) -> Option<&T> {
        todo!("Exercise 7")
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        todo!("Exercise 7")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 7")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 7")
    }

    /// Borrowing iterator, top of the stack first.
    pub fn iter(&self) -> std::iter::Rev<std::slice::Iter<'_, T>> {
        todo!("Exercise 7")
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        todo!("Exercise 7")
    }
}

/// Bonus: `for x in stack` -- consumes the stack, yields top first (the order
/// you'd get by popping). Note how much is just naming the iterator type.
impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = std::iter::Rev<std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        todo!("Exercise 7")
    }
}

/// And `for x in &stack` -- borrows.
impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        todo!("Exercise 7")
    }
}

/// And `stack.extend(..)` / `iter.collect::<Stack<_>>()`.
impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        todo!("Exercise 7")
    }
}

pub fn run() {
    todo!("Exercise 7")
}
