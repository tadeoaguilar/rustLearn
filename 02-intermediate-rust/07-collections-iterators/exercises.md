# Exercises: Collections & Iterators

## Exercise 1: Vector Operations

**Difficulty**: Easy
**Time**: 20 minutes

Master Vec<T> operations.

```rust
fn main() {
    // Create vectors
    let mut v1: Vec<i32> = Vec::new();
    let v2 = vec![1, 2, 3, 4, 5];

    // Add elements
    v1.push(1);
    v1.push(2);
    v1.extend([3, 4, 5]);

    // Access elements
    let third = &v2[2];
    let third_option = v2.get(2);

    match v2.get(10) {
        Some(value) => println!("Value: {}", value),
        None => println!("No value at index 10"),
    }

    // Iterate
    for i in &v2 {
        println!("{}", i);
    }

    // Modify elements
    for i in &mut v1 {
        *i *= 2;
    }
}
```

**Tasks**:
1. Create a function that removes duplicates from a Vec
2. Implement a function to split a Vec at a specific value
3. Merge two sorted vectors into one sorted vector

---

## Exercise 2: HashMap Basics

**Difficulty**: Medium
**Time**: 30 minutes

Work with HashMaps effectively.

```rust
use std::collections::HashMap;

fn main() {
    // Create and populate
    let mut scores = HashMap::new();
    scores.insert("Blue", 10);
    scores.insert("Red", 50);

    // Access values
    let team_name = "Blue";
    if let Some(score) = scores.get(team_name) {
        println!("{}: {}", team_name, score);
    }

    // Iterate
    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }

    // Update values
    scores.entry("Blue").or_insert(0);  // Insert if not exists
    *scores.entry("Blue").or_insert(0) += 10;  // Update existing
}
```

**Tasks**:
1. Word frequency counter
2. Group people by age from a list
3. Implement a simple cache with get/set/evict

---

## Exercise 3: Iterator Basics

**Difficulty**: Medium
**Time**: 35 minutes

Master iterator fundamentals.

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    // Consuming adapters
    let sum: i32 = v.iter().sum();
    let product: i32 = v.iter().product();

    // Iterator adaptors (lazy)
    let doubled: Vec<i32> = v.iter()
        .map(|x| x * 2)
        .collect();

    let evens: Vec<&i32> = v.iter()
        .filter(|x| *x % 2 == 0)
        .collect();

    // Chain operations
    let result: Vec<i32> = v.iter()
        .filter(|x| *x % 2 == 0)
        .map(|x| x * x)
        .collect();

    // take, skip, enumerate
    let first_three: Vec<&i32> = v.iter().take(3).collect();
    let skip_two: Vec<&i32> = v.iter().skip(2).collect();

    for (index, value) in v.iter().enumerate() {
        println!("{}: {}", index, value);
    }
}
```

**Tasks**:
1. Sum of squares of even numbers
2. Find first number divisible by 3 and 5
3. Flatten nested vectors

---

## Exercise 4: Advanced Iterators

**Difficulty**: Hard
**Time**: 45 minutes

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // fold - accumulator pattern
    let sum = numbers.iter().fold(0, |acc, x| acc + x);

    // scan - stateful transformation
    let running_sum: Vec<i32> = numbers.iter()
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect();

    // zip - combine iterators
    let a = vec![1, 2, 3];
    let b = vec!["one", "two", "three"];
    let combined: Vec<(i32, &str)> = a.iter()
        .copied()
        .zip(b.iter().copied())
        .collect();

    // flat_map - map and flatten
    let nested = vec![vec![1, 2], vec![3, 4]];
    let flat: Vec<i32> = nested.iter()
        .flat_map(|v| v.iter())
        .copied()
        .collect();
}
```

**Tasks**:
1. Implement Fibonacci using iterators
2. Cartesian product of two vectors
3. Partition numbers into even and odd

---

## Exercise 5: Custom Iterator

**Difficulty**: Hard
**Time**: 60 minutes

Implement your own iterator.

```rust
struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr = self.next;
        self.next = current + self.next;
        Some(current)
    }
}

fn main() {
    let fib = Fibonacci::new();

    // First 10 Fibonacci numbers
    for num in fib.take(10) {
        println!("{}", num);
    }
}
```

**Tasks**:
1. Create a Range-like iterator
2. Implement an iterator that cycles through a slice
3. Build an iterator that generates prime numbers

---

## Exercise 6: Closures

**Difficulty**: Medium
**Time**: 35 minutes

Master closure types: Fn, FnMut, FnOnce.

```rust
fn main() {
    let x = 5;

    // Fn - borrows immutably
    let add_x = |y| x + y;
    println!("{}", add_x(3));
    println!("{}", add_x(4));  // Can call multiple times

    // FnMut - borrows mutably
    let mut count = 0;
    let mut increment = || {
        count += 1;
        count
    };
    println!("{}", increment());
    println!("{}", increment());

    // FnOnce - takes ownership
    let message = String::from("Hello");
    let consume = || {
        let _temp = message;  // Takes ownership
    };
    consume();
    // consume();  // Error! Can only call once
}
```

**Tasks**:
1. Implement a function that takes a closure and applies it
2. Create a counter using closures
3. Build a function composition system

---

## Bonus Challenge: Data Pipeline

**Difficulty**: Very Hard
**Time**: 90+ minutes

Build a data processing pipeline.

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Sale {
    product: String,
    amount: f64,
    quantity: i32,
}

fn main() {
    let sales = vec![
        Sale {
            product: "Widget".to_string(),
            amount: 10.0,
            quantity: 5,
        },
        Sale {
            product: "Gadget".to_string(),
            amount: 15.0,
            quantity: 3,
        },
        Sale {
            product: "Widget".to_string(),
            amount: 10.0,
            quantity: 2,
        },
    ];

    // Total revenue by product
    let revenue_by_product: HashMap<String, f64> = sales
        .iter()
        .map(|sale| (sale.product.clone(), sale.amount * sale.quantity as f64))
        .fold(HashMap::new(), |mut acc, (product, revenue)| {
            *acc.entry(product).or_insert(0.0) += revenue;
            acc
        });

    println!("{:?}", revenue_by_product);

    // Top 3 products by revenue
    let mut products: Vec<_> = revenue_by_product.iter().collect();
    products.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let top_3: Vec<_> = products.iter().take(3).collect();

    println!("Top 3: {:?}", top_3);
}
```

**Tasks**:
1. Filter sales above a threshold
2. Calculate average sale amount per product
3. Find products with total quantity > 10
4. Group sales by price ranges

---

## Check Your Understanding

- [ ] Work with Vec, HashMap, HashSet
- [ ] Use iterator adaptors (map, filter, fold)
- [ ] Understand lazy vs eager evaluation
- [ ] Create custom iterators
- [ ] Use closures effectively
- [ ] Understand Fn, FnMut, FnOnce
- [ ] Chain iterator operations
- [ ] Optimize collection usage

---

## Additional Resources

- [Rust Book Chapter 8](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
- [Rust Book Chapter 13](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
- [Iterator Docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
