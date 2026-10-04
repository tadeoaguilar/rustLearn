//! Exercise 7: Generic Data Structures -- a Stack<T>.

#[derive(Debug, Clone, PartialEq)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Borrowing iterator, top of the stack first.
    pub fn iter(&self) -> std::iter::Rev<std::slice::Iter<'_, T>> {
        self.items.iter().rev()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Bonus: `for x in stack` -- consumes the stack, yields top first (the order
/// you'd get by popping). Note how much is just naming the iterator type.
impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = std::iter::Rev<std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter().rev()
    }
}

/// And `for x in &stack` -- borrows.
impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Rev<std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// And `stack.extend(..)` / `iter.collect::<Stack<_>>()`.
impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Stack {
            items: iter.into_iter().collect(),
        }
    }
}

pub fn run() {
    let mut stack = Stack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.peek(), Some(&2));
    assert_eq!(stack.len(), 2);
    println!("after pushes and a pop: {stack:?}");

    let stack: Stack<&str> = ["bottom", "middle", "top"].into_iter().collect();
    for item in &stack {
        print!("{item} ");
    }
    println!("(borrowed, top first)");
    let owned: Vec<&str> = stack.into_iter().collect();
    println!("consumed: {owned:?}");
}
