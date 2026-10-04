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
        Stack { head: None, len: 0 }
    }

    pub fn push(&mut self, value: T) {
        // take() moves the old head out, leaving None, so we can put it
        // inside the new node without two owners existing at once.
        let next = self.head.take();
        self.head = Some(Box::new(Node { value, next }));
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.len -= 1;
            node.value // the Box is freed here; we keep only the value
        })
    }

    pub fn peek(&self) -> Option<&T> {
        // as_deref: &Option<Box<Node>> -> Option<&Node>
        self.head.as_deref().map(|node| &node.value)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_deref_mut().map(|node| &mut node.value)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Reverses in place by re-pointing every `next`. No allocation, no clone.
    pub fn reverse(&mut self) {
        let mut previous: Link<T> = None;
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
            node.next = previous;
            previous = Some(node);
        }
        self.head = previous;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Without this, dropping `head` drops its Box, which drops `next`, which
/// drops *its* Box... one stack frame per node. A million nodes overflow the
/// stack. The loop below unlinks each node before it is dropped, so each drop
/// is shallow.
impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
            // `node` is dropped here with next == None: no recursion.
        }
    }
}

/// Borrowing iterator: holds a reference to the next node to visit.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.value
        })
    }
}

/// Consuming iterator: just pops.
pub struct IntoIter<T>(Stack<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> FromIterator<T> for Stack<T> {
    /// Pushes in order, so the *last* item ends up on top.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut s = Stack::new();
        for x in iter {
            s.push(x);
        }
        s
    }
}

pub fn run() {
    let mut s: Stack<i32> = (1..=5).collect();
    println!("top first: {:?}", s.iter().collect::<Vec<_>>());
    s.reverse();
    println!("reversed:  {:?}", s.iter().collect::<Vec<_>>());
    if let Some(top) = s.peek_mut() {
        *top *= 100;
    }
    println!("pop -> {:?}, len now {}", s.pop(), s.len());

    let mut big = Stack::new();
    for i in 0..1_000_000 {
        big.push(i);
    }
    drop(big);
    println!("dropped a 1,000,000-node list without overflowing the stack");
}
