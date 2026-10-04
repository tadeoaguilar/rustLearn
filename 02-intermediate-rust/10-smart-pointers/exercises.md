# Exercises: Smart Pointers

## Exercise 1: Box and Recursive Types

**Difficulty**: Easy
**Time**: 25 minutes

**Learning Objectives**:
- Understand why recursive types need indirection
- Use `Box<T>` to put a value on the heap

**Task 1**: The cons list
```rust
#[derive(Debug, PartialEq)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

fn sum(list: &List) -> i32 {
    // Recursively add up every value
}

fn from_slice(values: &[i32]) -> List {
    // [1, 2, 3] -> Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))))
}
```

Try removing the `Box` and read the compiler error (`E0072: recursive type has
infinite size`).

**Task 2**: An expression tree
```rust
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

fn eval(expr: &Expr) -> f64 { ... }
fn to_string(expr: &Expr) -> String { ... }   // (1 + 2) * -3
```

**Tests**:
```rust
let e = Expr::Mul(
    Box::new(Expr::Add(Box::new(Expr::Num(1.0)), Box::new(Expr::Num(2.0)))),
    Box::new(Expr::Neg(Box::new(Expr::Num(3.0)))),
);
assert_eq!(eval(&e), -9.0);
assert_eq!(to_string(&e), "((1 + 2) * -3)");
```

---

## Exercise 2: A Linked List with Box

**Difficulty**: Medium
**Time**: 60 minutes

**Learning Objectives**:
- Own a chain of heap nodes through `Option<Box<Node<T>>>`
- Use `Option::take`, `as_ref`, `as_deref`, `as_mut`
- Write an iterative `Drop` to avoid a stack overflow

**Task**: Implement a singly linked stack.

```rust
pub struct Stack<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Stack<T> {
    pub fn new() -> Self;
    pub fn push(&mut self, value: T);
    pub fn pop(&mut self) -> Option<T>;
    pub fn peek(&self) -> Option<&T>;
    pub fn peek_mut(&mut self) -> Option<&mut T>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn reverse(&mut self);              // in place, no allocation
    pub fn iter(&self) -> Iter<'_, T>;      // borrowing iterator
}

impl<T> Drop for Stack<T> { ... }           // see below
```

**Why a custom `Drop`?** The default drop of a `Box<Node>` drops its `next`,
which drops *its* `next`... recursively. A list of a million nodes overflows
the stack. Write a loop that unlinks nodes one at a time.

**Test**:
```rust
let mut s = Stack::new();
for i in 0..1_000_000 { s.push(i); }
drop(s);   // must not overflow the stack
```

**Hints**:
- `self.head.take()` moves the head out and leaves `None` behind
- `self.head.as_deref()` turns `&Option<Box<Node<T>>>` into `Option<&Node<T>>`
- Read [Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)

---

## Exercise 3: Deref and Drop

**Difficulty**: Medium
**Time**: 35 minutes

**Learning Objectives**:
- Implement `Deref` and see deref coercion in action
- Implement `Drop` and observe drop order

**Task 1**: `MyBox<T>`
```rust
struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

fn hello(name: &str) -> String { format!("Hello, {name}!") }

let m = MyBox::new(String::from("Rust"));
assert_eq!(hello(&m), "Hello, Rust!");   // &MyBox<String> -> &String -> &str
```

**Task 2**: A drop logger
```rust
struct Noisy { name: String, log: Rc<RefCell<Vec<String>>> }

impl Drop for Noisy {
    fn drop(&mut self) {
        self.log.borrow_mut().push(format!("drop {}", self.name));
    }
}
```

Write tests that prove:
- locals are dropped in **reverse** order of declaration
- struct fields are dropped in **declaration** order
- `std::mem::drop(x)` drops early
- moving a value into a function moves its drop to the end of that function

---

## Exercise 4: Rc and Weak — a Tree with Parent Links

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Share ownership with `Rc<T>`
- Break reference cycles with `Weak<T>`
- Inspect `Rc::strong_count` and `Rc::weak_count`

**Task**: A tree where children are owned by their parent, and each child can
find its parent without owning it.

```rust
pub struct TreeNode {
    pub value: String,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    pub fn new(value: &str) -> Rc<TreeNode>;
    pub fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>);
    pub fn parent(&self) -> Option<Rc<TreeNode>>;
    pub fn depth(&self) -> usize;                 // root = 0
    pub fn path(&self) -> String;                 // "root/usr/bin"
    pub fn find(node: &Rc<TreeNode>, value: &str) -> Option<Rc<TreeNode>>;
}
```

**Questions**:
- If the parent link were `Rc` instead of `Weak`, what would happen when you drop the root?
- After dropping the root, what does `child.parent()` return?

---

## Exercise 5: Interior Mutability with RefCell and Cell

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Mutate through a shared reference with `RefCell<T>` and `Cell<T>`
- Understand runtime borrow checking, and avoid its panics

**Task 1**: The Rust Book's `LimitTracker` with a test double
```rust
pub trait Messenger {
    fn send(&self, msg: &str);   // &self, not &mut self!
}

pub struct LimitTracker<'a, T: Messenger> { messenger: &'a T, value: usize, max: usize }

impl<'a, T: Messenger> LimitTracker<'a, T> {
    pub fn set_value(&mut self, value: usize) {
        // >= 100%: "Error: You are over your quota!"
        // >= 90%:  "Urgent warning: You've used up over 90% of your quota!"
        // >= 75%:  "Warning: You've used up over 75% of your quota!"
    }
}
```

Write a `MockMessenger` that records messages in a `RefCell<Vec<String>>`
even though `send` only gets `&self`.

**Task 2**: `Cell<T>` for `Copy` values — count how many times `send` is
called with a `Cell<usize>`.

**Task 3**: Make a double `borrow_mut()` panic in a test (`#[should_panic]`),
then avoid the panic with `try_borrow_mut()`.

---

## Exercise 6: Shared Mutable Graphs with Rc<RefCell<T>>

**Difficulty**: Hard
**Time**: 50 minutes

**Learning Objectives**:
- Combine `Rc` (shared ownership) and `RefCell` (shared mutation)
- See a reference cycle leak, then fix it

**Task 1**: A social network.
```rust
pub type PersonRef = Rc<RefCell<Person>>;

pub struct Person {
    pub name: String,
    friends: Vec<Weak<RefCell<Person>>>,
}

pub fn befriend(a: &PersonRef, b: &PersonRef);   // both directions
pub fn friend_names(p: &PersonRef) -> Vec<String>;
pub fn mutual_friends(a: &PersonRef, b: &PersonRef) -> Vec<String>;
pub fn reachable(from: &PersonRef) -> Vec<String>;  // everyone connected, BFS
```

**Task 2**: Demonstrate a leak. Build two nodes that hold `Rc`s to each other,
drop both variables, and show (with a `Weak` you kept, or a drop counter) that
the nodes were never freed. Then explain why the `Weak` friends list above
doesn't leak.

---

## Exercise 7: A Thread-Safe Cache with Arc and Mutex

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Share data between threads with `Arc<T>`
- Mutate it safely with `Mutex<T>` and `RwLock<T>`
- Understand why `Rc` and `RefCell` don't compile across threads

**Task**:
```rust
#[derive(Clone)]
pub struct SharedCache<K, V> {
    inner: Arc<Mutex<HashMap<K, V>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> SharedCache<K, V> {
    pub fn new() -> Self;
    pub fn get(&self, key: &K) -> Option<V>;
    pub fn insert(&self, key: K, value: V);
    pub fn get_or_compute(&self, key: K, compute: impl FnOnce() -> V) -> V;
    pub fn len(&self) -> usize;
}
```

Requirements:
- `get_or_compute` from 8 threads with the same key must run `compute`
  **exactly once** (count calls with an `AtomicUsize`)
- Every method takes `&self` — the cache is shared, not owned
- Hold the lock for as short a time as possible in `get`

Then write a second version with `RwLock` so many readers can `get` at once.

**Questions**:
- Try `Rc<RefCell<HashMap>>` and `thread::spawn`. What's the error, and what
  do `Send` and `Sync` mean?
- What happens to a `Mutex` if a thread panics while holding it?

---

## Bonus Challenge: Cow — Clone on Write

**Difficulty**: Medium
**Time**: 30 minutes

```rust
use std::borrow::Cow;

/// Collapses runs of whitespace into one space. Allocates only if it has to.
fn normalize_whitespace(s: &str) -> Cow<'_, str>;

assert!(matches!(normalize_whitespace("already fine"), Cow::Borrowed(_)));
assert!(matches!(normalize_whitespace("too   many"), Cow::Owned(_)));
```

---

## Which Pointer When?

| You need | Use |
|---|---|
| A value on the heap, one owner (recursive types, big values, trait objects) | `Box<T>` |
| Several owners, one thread | `Rc<T>` |
| Several owners, many threads | `Arc<T>` |
| Mutation through `&self`, one thread, `Copy` values | `Cell<T>` |
| Mutation through `&self`, one thread | `RefCell<T>` |
| Mutation through `&self`, many threads | `Mutex<T>` / `RwLock<T>` / atomics |
| A back-reference that mustn't keep things alive | `Weak<T>` |
| Borrow if possible, own if necessary | `Cow<'a, T>` |

---

## Check Your Understanding

- [ ] Explain why `enum List { Cons(i32, List), Nil }` doesn't compile
- [ ] Write a linked list with `Box` and a non-recursive `Drop`
- [ ] Implement `Deref` and explain deref coercion
- [ ] Build a tree with `Rc` children and `Weak` parents
- [ ] Use `RefCell` and know when it panics
- [ ] Share mutable state across threads with `Arc<Mutex<T>>`

---

## Additional Resources

- [Rust Book Chapter 15](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Rust Book Chapter 16.3: Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
- [Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
