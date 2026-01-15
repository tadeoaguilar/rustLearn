# Exercises: Traits & Generics

## Exercise 1: Basic Traits

**Difficulty**: Easy
**Time**: 25 minutes

Define and implement basic traits.

**Task 1**: Summary trait
```rust
trait Summary {
    fn summarize(&self) -> String;
}

struct Article {
    title: String,
    author: String,
    content: String,
}

struct Tweet {
    username: String,
    content: String,
    reply: bool,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.content)
    }
}

fn main() {
    let article = Article {
        title: "Rust is Great".to_string(),
        author: "Alice".to_string(),
        content: "Rust provides...".to_string(),
    };

    let tweet = Tweet {
        username: "bob".to_string(),
        content: "Learning Rust!".to_string(),
        reply: false,
    };

    println!("{}", article.summarize());
    println!("{}", tweet.summarize());
}
```

**Task 2**: Default trait implementations
```rust
trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // Uses default summarize implementation
}
```

---

## Exercise 2: Generic Functions

**Difficulty**: Medium
**Time**: 30 minutes

**Task 1**: Generic largest function
```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Largest: {}", largest(&numbers));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest: {}", largest(&chars));
}
```

**Task 2**: Generic swap function
```rust
fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

fn main() {
    let mut x = 5;
    let mut y = 10;
    swap(&mut x, &mut y);
    assert_eq!(x, 10);
    assert_eq!(y, 5);
}
```

**Task 3**: Multiple generic types
```rust
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }

    fn get_first(&self) -> &T {
        &self.first
    }

    fn get_second(&self) -> &U {
        &self.second
    }
}

fn main() {
    let pair = Pair::new(5, "hello");
    println!("{}, {}", pair.get_first(), pair.get_second());
}
```

---

## Exercise 3: Trait Bounds

**Difficulty**: Medium
**Time**: 35 minutes

**Task 1**: Function with trait bounds
```rust
use std::fmt::Display;

fn print_pair<T: Display, U: Display>(a: T, b: U) {
    println!("{} and {}", a, b);
}

fn print_pair_where<T, U>(a: T, b: U)
where
    T: Display,
    U: Display,
{
    println!("{} and {}", a, b);
}
```

**Task 2**: Implement trait conditionally
```rust
struct Pair<T> {
    x: T,
    y: T,
}

impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Pair { x, y }
    }
}

// Only implement for types that are Display + PartialOrd
impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("Largest: {}", self.x);
        } else {
            println!("Largest: {}", self.y);
        }
    }
}
```

**Task 3**: Multiple trait bounds
```rust
fn compare_and_print<T>(a: T, b: T) -> T
where
    T: PartialOrd + Display + Clone,
{
    if a > b {
        println!("{} is greater", a);
        a
    } else {
        println!("{} is greater", b);
        b.clone()
    }
}
```

---

## Exercise 4: Operator Overloading

**Difficulty**: Medium
**Time**: 30 minutes

Implement standard traits for custom types.

**Task**: Vector2D with operator overloading
```rust
use std::ops::Add;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector2D {
    x: f64,
    y: f64,
}

impl Vector2D {
    fn new(x: f64, y: f64) -> Self {
        Vector2D { x, y }
    }

    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Implement Add trait for + operator
impl Add for Vector2D {
    type Output = Vector2D;

    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Implement Display trait
impl fmt::Display for Vector2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// TODO: Implement Sub, Mul (scalar), Neg traits

fn main() {
    let v1 = Vector2D::new(1.0, 2.0);
    let v2 = Vector2D::new(3.0, 4.0);
    let v3 = v1 + v2;

    println!("{} + {} = {}", v1, v2, v3);
    assert_eq!(v3, Vector2D::new(4.0, 6.0));
}
```

---

## Exercise 5: Associated Types

**Difficulty**: Hard
**Time**: 40 minutes

**Task**: Implement Iterator trait
```rust
struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Counter {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;  // Associated type

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

fn main() {
    let counter = Counter::new(5);

    for num in counter {
        println!("{}", num);
    }

    // Outputs: 1, 2, 3, 4, 5
}
```

**Task 2**: Graph trait with associated types
```rust
trait Graph {
    type Node;
    type Edge;

    fn has_edge(&self, from: &Self::Node, to: &Self::Node) -> bool;
    fn edges(&self, node: &Self::Node) -> Vec<&Self::Edge>;
}

// Implement for a specific graph type
```

---

## Exercise 6: Trait Objects

**Difficulty**: Hard
**Time**: 45 minutes

**Task**: Plugin system with trait objects
```rust
trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self);
}

struct LoggerPlugin {
    log_level: String,
}

struct CachePlugin {
    cache_size: usize,
}

impl Plugin for LoggerPlugin {
    fn name(&self) -> &str {
        "Logger"
    }

    fn execute(&self) {
        println!("Logging at level: {}", self.log_level);
    }
}

impl Plugin for CachePlugin {
    fn name(&self) -> &str {
        "Cache"
    }

    fn execute(&self) {
        println!("Cache size: {} MB", self.cache_size);
    }
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    fn run_all(&self) {
        for plugin in &self.plugins {
            println!("Running plugin: {}", plugin.name());
            plugin.execute();
        }
    }
}

fn main() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(LoggerPlugin {
        log_level: "INFO".to_string(),
    }));

    manager.register(Box::new(CachePlugin { cache_size: 128 }));

    manager.run_all();
}
```

---

## Exercise 7: Generic Data Structures

**Difficulty**: Hard
**Time**: 60 minutes

**Task**: Implement a generic Stack
```rust
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

fn main() {
    let mut stack = Stack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);

    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.peek(), Some(&2));
    assert_eq!(stack.len(), 2);
}
```

**Bonus**: Implement IntoIterator for Stack

---

## Bonus Challenge: Shape System

**Difficulty**: Very Hard
**Time**: 90+ minutes

Build a complete shape system with traits and generics.

```rust
use std::fmt;

trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

trait Drawable: Shape {
    fn draw(&self);
}

#[derive(Debug, Clone)]
struct Circle {
    radius: f64,
}

#[derive(Debug, Clone)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

fn total_area<T: Shape>(shapes: &[T]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn draw_all(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();
    }
}

fn main() {
    let circles = vec![
        Circle { radius: 1.0 },
        Circle { radius: 2.0 },
    ];

    println!("Total circle area: {}", total_area(&circles));

    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 3.0 }),
        Box::new(Rectangle { width: 4.0, height: 5.0 }),
    ];

    draw_all(&shapes);
}
```

---

## Check Your Understanding

- [ ] Define and implement traits
- [ ] Use generic type parameters
- [ ] Apply trait bounds to generics
- [ ] Implement operator traits (Add, Sub, etc.)
- [ ] Use associated types in traits
- [ ] Work with trait objects (dyn Trait)
- [ ] Understand static vs dynamic dispatch
- [ ] Create generic data structures

---

## Additional Resources

- [Rust Book Chapter 10](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
