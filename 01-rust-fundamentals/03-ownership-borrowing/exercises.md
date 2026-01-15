# Exercises: Ownership & Borrowing

## Exercise 1: Understanding Ownership

**Difficulty**: Easy
**Time**: 20 minutes

Understand basic ownership rules.

**Task 1**: Fix the ownership errors
```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is moved to s2

    // This will error - s1 is no longer valid
    // println!("{}", s1);

    // Fix this code in three different ways:
    // 1. Using clone()
    // 2. Using a reference
    // 3. Restructuring to avoid the issue
}
```

**Task 2**: Ownership with functions
```rust
fn takes_ownership(s: String) {
    println!("{}", s);
} // s is dropped here

fn main() {
    let s = String::from("hello");
    takes_ownership(s);
    // Can't use s here anymore!
    // println!("{}", s);  // Error!

    // How can you fix this to use s after the function call?
}
```

**Questions**:
- What happens when a value goes out of scope?
- What's the difference between move and copy semantics?
- Which types implement the Copy trait?

---

## Exercise 2: References and Borrowing

**Difficulty**: Medium
**Time**: 30 minutes

Master borrowing rules.

**Task 1**: Immutable references
```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}  // s goes out of scope, but nothing is dropped

fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("The length of '{}' is {}.", s1, len);
    // s1 is still valid!
}
```

**Task 2**: Mutable references
```rust
fn append_world(s: &mut String) {
    s.push_str(", world");
}

fn main() {
    let mut s = String::from("hello");
    append_world(&mut s);
    println!("{}", s);
}
```

**Task 3**: Borrowing rules
```rust
fn main() {
    let mut s = String::from("hello");

    // Fix this code to follow borrowing rules
    let r1 = &s;
    let r2 = &s;
    let r3 = &mut s;  // Error! Can't have mutable ref while immutable refs exist

    println!("{}, {}, and {}", r1, r2, r3);
}
```

**Hint**: Remember that a reference's scope ends at its last use, not necessarily at the end of the block.

---

## Exercise 3: String vs &str

**Difficulty**: Medium
**Time**: 25 minutes

Understand the difference between String and string slices.

**Task 1**: Implement these functions
```rust
// Return the first word from a sentence
fn first_word(s: &str) -> &str {
    // Find the first space, return slice up to that point
    // If no space, return the entire string
}

// Return the last word from a sentence
fn last_word(s: &str) -> &str {
    // Similar to first_word but from the end
}

// Reverse a string (return a new String)
fn reverse_string(s: &str) -> String {
    // Return a new String with characters reversed
}

fn main() {
    let sentence = String::from("Hello Rust World");

    assert_eq!(first_word(&sentence), "Hello");
    assert_eq!(last_word(&sentence), "World");
    assert_eq!(reverse_string("hello"), "olleh");
}
```

**Task 2**: String concatenation
```rust
fn main() {
    // Implement string concatenation in 4 different ways:
    // 1. Using + operator
    // 2. Using format! macro
    // 3. Using push_str method
    // 4. Using String::from with multiple operations
}
```

---

## Exercise 4: Dangling References

**Difficulty**: Medium
**Time**: 20 minutes

Understand how Rust prevents dangling references.

**Task 1**: Fix the dangling reference
```rust
// This code doesn't compile - fix it!
fn dangle() -> &String {
    let s = String::from("hello");
    &s  // Error! s will be dropped, reference would be dangling
}

// Solution 1: Return the String itself

// Solution 2: Use 'static str
```

**Task 2**: Reference lifetime
```rust
fn longest(x: &str, y: &str) -> &str {
    // Return the longer string
    // This won't compile yet - you'll fix it in lifetimes module
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

---

## Exercise 5: Clone vs Copy

**Difficulty**: Medium
**Time**: 30 minutes

Understand the difference between Clone and Copy.

**Task 1**: Identify Copy types
```rust
fn main() {
    // Which of these are Copy and which require clone?
    let x = 5;
    let y = x;  // Copy (i32 is Copy)

    let s1 = String::from("hello");
    let s2 = s1;  // Move (String is NOT Copy)

    // Test these types:
    let a: i32 = 42;
    let b: f64 = 3.14;
    let c: bool = true;
    let d: char = 'a';
    let e: &str = "hello";
    let f: String = String::from("hello");
    let g: Vec<i32> = vec![1, 2, 3];
    let h: (i32, String) = (1, String::from("hello"));

    // For each, determine if it's Copy or needs clone
}
```

**Task 2**: Implement a Copy type
```rust
#[derive(Copy, Clone, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p1 = Point { x: 5, y: 10 };
    let p2 = p1;  // Copy, not move!
    println!("p1: {:?}, p2: {:?}", p1, p2);  // Both valid
}
```

**Task 3**: When you can't use Copy
```rust
// This won't work - explain why
// #[derive(Copy, Clone)]
struct Person {
    name: String,  // String doesn't implement Copy
    age: u32,
}

// Implement manual clone instead
```

---

## Exercise 6: Mutable vs Immutable Borrow

**Difficulty**: Hard
**Time**: 35 minutes

Practice complex borrowing scenarios.

**Task 1**: Multiple readers
```rust
fn main() {
    let s = String::from("hello");

    // Multiple immutable references are OK
    let r1 = &s;
    let r2 = &s;
    let r3 = &s;

    println!("{}, {}, {}", r1, r2, r3);
}
```

**Task 2**: One writer
```rust
fn main() {
    let mut s = String::from("hello");

    // Only one mutable reference at a time
    let r1 = &mut s;
    // let r2 = &mut s;  // Error!

    println!("{}", r1);
}
```

**Task 3**: Mix readers and writers (advanced)
```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;     // OK
    let r2 = &s;     // OK
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point

    let r3 = &mut s; // OK - no more immutable refs active
    r3.push_str(" world");
    println!("{}", r3);
}
```

**Task 4**: Build a safe cache
```rust
use std::collections::HashMap;

struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: HashMap::new(),
        }
    }

    // Immutable borrow - read access
    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    // Mutable borrow - write access
    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    // Can we call get and set at the same time?
    // Try it and understand the error
}

fn main() {
    let mut cache = Cache::new();
    cache.set("name".to_string(), "Alice".to_string());

    if let Some(name) = cache.get("name") {
        println!("Name: {}", name);
    }
}
```

---

## Exercise 7: Real-World Example - Text Buffer

**Difficulty**: Hard
**Time**: 45 minutes

Build a simple text buffer with ownership rules.

**Requirements**:
```rust
struct TextBuffer {
    content: String,
}

impl TextBuffer {
    fn new() -> Self {
        // Create empty buffer
    }

    fn from_str(s: &str) -> Self {
        // Create from string slice
    }

    fn append(&mut self, text: &str) {
        // Add text to end
    }

    fn prepend(&mut self, text: &str) {
        // Add text to beginning
    }

    fn clear(&mut self) {
        // Clear all content
    }

    fn len(&self) -> usize {
        // Return length
    }

    fn is_empty(&self) -> bool {
        // Check if empty
    }

    fn content(&self) -> &str {
        // Return reference to content
    }

    fn search(&self, needle: &str) -> Vec<usize> {
        // Return indices where needle appears
    }

    fn replace(&mut self, from: &str, to: &str) {
        // Replace all occurrences
    }
}

fn main() {
    let mut buffer = TextBuffer::new();
    buffer.append("Hello ");
    buffer.append("World");

    assert_eq!(buffer.content(), "Hello World");
    assert_eq!(buffer.len(), 11);

    buffer.prepend("Say: ");
    assert_eq!(buffer.content(), "Say: Hello World");

    let positions = buffer.search("l");
    assert_eq!(positions, vec![9, 13]);

    buffer.replace("World", "Rust");
    assert_eq!(buffer.content(), "Say: Hello Rust");

    buffer.clear();
    assert!(buffer.is_empty());
}
```

---

## Exercise 8: Ownership in Collections

**Difficulty**: Medium
**Time**: 30 minutes

Understand ownership with vectors.

**Task 1**: Vector ownership
```rust
fn main() {
    let v = vec![
        String::from("hello"),
        String::from("world"),
    ];

    // Getting elements from vector
    let first = &v[0];  // Borrow
    // let first = v[0];  // Error! Can't move out of vector

    // Iterating with references
    for s in &v {
        println!("{}", s);
    }
    // v is still valid here

    // Taking ownership
    for s in v {
        println!("{}", s);
    }
    // v is no longer valid here
}
```

**Task 2**: Modify vector elements
```rust
fn main() {
    let mut numbers = vec![1, 2, 3, 4, 5];

    // Modify in place
    for n in &mut numbers {
        *n *= 2;
    }

    assert_eq!(numbers, vec![2, 4, 6, 8, 10]);
}
```

**Task 3**: Remove from vector
```rust
fn remove_negatives(numbers: &mut Vec<i32>) {
    // Remove all negative numbers from the vector
    // Hint: use retain method or drain_filter
}

fn main() {
    let mut nums = vec![1, -2, 3, -4, 5];
    remove_negatives(&mut nums);
    assert_eq!(nums, vec![1, 3, 5]);
}
```

---

## Bonus Challenge: Reference Counting Simulation

**Difficulty**: Very Hard
**Time**: 60+ minutes

Simulate reference counting without using Rc.

**Requirements**:
Build a simple reference counting system using unsafe Rust (preview of advanced topics).

```rust
struct SimpleRc<T> {
    data: *mut RcBox<T>,
}

struct RcBox<T> {
    value: T,
    ref_count: usize,
}

impl<T> SimpleRc<T> {
    fn new(value: T) -> Self {
        // Allocate on heap with ref_count = 1
        todo!()
    }

    fn clone(&self) -> Self {
        // Increment ref_count
        todo!()
    }

    fn get(&self) -> &T {
        // Return reference to value
        todo!()
    }
}

impl<T> Drop for SimpleRc<T> {
    fn drop(&mut self) {
        // Decrement ref_count
        // If ref_count reaches 0, deallocate
        todo!()
    }
}

// Test
fn main() {
    let rc1 = SimpleRc::new(String::from("hello"));
    let rc2 = rc1.clone();

    println!("{}", rc1.get());
    println!("{}", rc2.get());

    // Both rc1 and rc2 should be valid
}
```

Note: This exercise uses unsafe Rust. We'll cover this properly in advanced modules.

---

## Check Your Understanding

Before moving to the next module, ensure you can:

- [ ] Explain the three ownership rules
- [ ] Understand when values are moved vs copied
- [ ] Use references (&T and &mut T) correctly
- [ ] Follow the borrowing rules (one mutable OR many immutable)
- [ ] Understand the difference between String and &str
- [ ] Prevent dangling references
- [ ] Know which types are Copy vs Clone
- [ ] Work with ownership in collections

---

## Common Mistakes to Avoid

1. **Using value after move**: Can't use a moved value
2. **Multiple mutable references**: Only one &mut at a time
3. **Mixing mutable and immutable**: Can't have &mut while & exists
4. **Returning references to local variables**: Creates dangling reference
5. **Confusion about string types**: String vs &str vs &String

---

## Memory Safety Guarantees

Rust's ownership system guarantees:
- ✅ No use-after-free
- ✅ No double-free
- ✅ No dangling pointers
- ✅ No data races (in safe Rust)
- ✅ No null pointer dereferences

All at **compile time** with **zero runtime cost**!

---

## Additional Resources

- [Rust Book Chapter 4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Visualizing Memory Layout](https://fasterthanli.me/articles/i-am-a-java-csharp-c-or-cplusplus-dev-time-to-do-some-rust)
- [Ownership in Rust (video)](https://www.youtube.com/watch?v=VFIOSWy93H0)
