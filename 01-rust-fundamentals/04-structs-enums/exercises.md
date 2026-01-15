# Exercises: Structs & Enums

## Exercise 1: Basic Structs

**Difficulty**: Easy
**Time**: 20 minutes

Create and use structs.

**Task 1**: Define a User struct
```rust
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

fn main() {
    // Create a user instance
    let user1 = User {
        email: String::from("user@example.com"),
        username: String::from("user123"),
        active: true,
        sign_in_count: 1,
    };

    // Access fields
    println!("User: {}", user1.username);
}
```

**Task 2**: Mutable structs
```rust
fn main() {
    let mut user = User {
        email: String::from("test@example.com"),
        username: String::from("test"),
        active: true,
        sign_in_count: 1,
    };

    // Modify a field
    user.email = String::from("newemail@example.com");
}
```

**Task 3**: Struct update syntax
```rust
fn main() {
    let user1 = User { /* ... */ };

    // Create user2 with some fields from user1
    let user2 = User {
        email: String::from("another@example.com"),
        ..user1  // Take remaining fields from user1
    };

    // Can you still use user1? Why or why not?
}
```

---

## Exercise 2: Tuple Structs and Unit Structs

**Difficulty**: Easy
**Time**: 15 minutes

Work with different struct types.

**Task 1**: Tuple structs
```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    // Even though both have same data, they're different types!
    // Color and Point are not interchangeable

    // Access tuple struct elements
    println!("Red: {}, Green: {}, Blue: {}", black.0, black.1, black.2);
}
```

**Task 2**: Unit struct
```rust
struct AlwaysEqual;

fn main() {
    let subject = AlwaysEqual;
    // Useful for implementing traits without data
}
```

**Task 3**: Create coordinate system
```rust
struct Point2D(f64, f64);
struct Point3D(f64, f64, f64);

fn distance_from_origin(point: Point2D) -> f64 {
    // Calculate: sqrt(x² + y²)
}

fn main() {
    let p = Point2D(3.0, 4.0);
    assert_eq!(distance_from_origin(p), 5.0);
}
```

---

## Exercise 3: Methods and Associated Functions

**Difficulty**: Medium
**Time**: 35 minutes

Implement methods on structs.

**Task**: Rectangle calculator
```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // Associated function (constructor)
    fn new(width: u32, height: u32) -> Self {
        Rectangle { width, height }
    }

    // Another constructor - square
    fn square(size: u32) -> Self {
        Rectangle {
            width: size,
            height: size,
        }
    }

    // Method - takes &self
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // Method - check if it can hold another rectangle
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    // Method - modify self
    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }

    // Method - consuming self
    fn to_square(self) -> Rectangle {
        let size = std::cmp::max(self.width, self.height);
        Rectangle::square(size)
    }
}

fn main() {
    let rect1 = Rectangle::new(30, 50);
    let rect2 = Rectangle::square(25);

    println!("Area: {}", rect1.area());
    println!("Can hold: {}", rect1.can_hold(&rect2));

    let mut rect3 = Rectangle::new(10, 20);
    rect3.scale(2);
    assert_eq!(rect3.area(), 400);

    let square = rect3.to_square();
    // rect3 is no longer valid - it was consumed
}
```

---

## Exercise 4: Basic Enums

**Difficulty**: Easy
**Time**: 20 minutes

Create and use enums.

**Task 1**: Direction enum
```rust
enum Direction {
    North,
    South,
    East,
    West,
}

fn move_player(direction: Direction) -> (i32, i32) {
    // Return (x, y) delta for movement
    match direction {
        Direction::North => (0, 1),
        Direction::South => (0, -1),
        Direction::East => (1, 0),
        Direction::West => (-1, 0),
    }
}
```

**Task 2**: Enums with data
```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit"),
            Message::Move { x, y } => println!("Move to ({}, {})", x, y),
            Message::Write(text) => println!("Write: {}", text),
            Message::ChangeColor(r, g, b) => {
                println!("Change color to ({}, {}, {})", r, g, b)
            }
        }
    }
}

fn main() {
    let messages = vec![
        Message::Quit,
        Message::Move { x: 10, y: 20 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(255, 0, 0),
    ];

    for msg in messages {
        msg.call();
    }
}
```

---

## Exercise 5: Option<T>

**Difficulty**: Medium
**Time**: 30 minutes

Master the Option enum.

**Task 1**: Basic Option usage
```rust
fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Some(result) => println!("Result: {}", result),
        None => println!("Cannot divide by zero"),
    }
}
```

**Task 2**: Option methods
```rust
fn main() {
    let some_number = Some(5);
    let no_number: Option<i32> = None;

    // is_some() and is_none()
    assert!(some_number.is_some());
    assert!(no_number.is_none());

    // unwrap() - panics if None
    let value = some_number.unwrap();

    // unwrap_or() - provides default
    let value = no_number.unwrap_or(0);

    // unwrap_or_else() - lazy default
    let value = no_number.unwrap_or_else(|| expensive_computation());

    // map() - transform the value
    let doubled = some_number.map(|n| n * 2);

    // and_then() - chain operations
    let result = some_number
        .and_then(|n| if n > 3 { Some(n * 2) } else { None });
}

fn expensive_computation() -> i32 {
    42
}
```

**Task 3**: Find element in vector
```rust
fn find_first_even(numbers: &[i32]) -> Option<i32> {
    // Return the first even number, or None if not found
}

fn find_position<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    // Return the index of target, or None if not found
}

fn main() {
    let numbers = vec![1, 3, 5, 8, 9, 10];

    assert_eq!(find_first_even(&numbers), Some(8));
    assert_eq!(find_position(&numbers, &5), Some(2));
    assert_eq!(find_position(&numbers, &100), None);
}
```

---

## Exercise 6: Pattern Matching

**Difficulty**: Medium
**Time**: 35 minutes

Master pattern matching.

**Task 1**: Match with multiple patterns
```rust
fn describe_number(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1 | 2 => "one or two",
        3..=9 => "three through nine",
        10 => "ten",
        _ => "something else",
    }
}
```

**Task 2**: Destructuring structs
```rust
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 0, y: 7 };

    match point {
        Point { x: 0, y } => println!("On y-axis at {}", y),
        Point { x, y: 0 } => println!("On x-axis at {}", x),
        Point { x, y } => println!("At ({}, {})", x, y),
    }
}
```

**Task 3**: Match guards
```rust
enum Temperature {
    Celsius(i32),
    Fahrenheit(i32),
}

fn describe_temperature(temp: Temperature) -> &'static str {
    match temp {
        Temperature::Celsius(t) if t < 0 => "freezing in Celsius",
        Temperature::Celsius(t) if t > 30 => "hot in Celsius",
        Temperature::Fahrenheit(t) if t < 32 => "freezing in Fahrenheit",
        Temperature::Fahrenheit(t) if t > 86 => "hot in Fahrenheit",
        _ => "moderate temperature",
    }
}
```

**Task 4**: if let and while let
```rust
fn main() {
    let some_value = Some(3);

    // Instead of:
    match some_value {
        Some(3) => println!("three"),
        _ => (),
    }

    // Use if let:
    if let Some(3) = some_value {
        println!("three");
    }

    // while let example
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
}
```

---

## Exercise 7: Result<T, E>

**Difficulty**: Medium
**Time**: 30 minutes

Work with the Result type for error handling.

**Task 1**: Basic Result
```rust
fn parse_int(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse::<i32>()
}

fn main() {
    match parse_int("42") {
        Ok(n) => println!("Parsed: {}", n),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Task 2**: Custom Result type
```rust
#[derive(Debug)]
enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
}

fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

fn sqrt(x: f64) -> Result<f64, MathError> {
    if x < 0.0 {
        Err(MathError::NegativeSquareRoot)
    } else {
        Ok(x.sqrt())
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {}", result),
        Err(MathError::DivisionByZero) => println!("Cannot divide by zero"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

---

## Exercise 8: Complex Enum - Game State

**Difficulty**: Hard
**Time**: 60 minutes

Build a game state machine using enums.

**Requirements**:
```rust
#[derive(Debug, Clone)]
enum GameState {
    Menu,
    Playing { level: u32, score: u32, lives: u8 },
    Paused { level: u32, score: u32, lives: u8 },
    GameOver { final_score: u32 },
}

impl GameState {
    fn new() -> Self {
        GameState::Menu
    }

    fn start_game(&mut self) {
        // Transition from Menu to Playing
        *self = GameState::Playing {
            level: 1,
            score: 0,
            lives: 3,
        };
    }

    fn pause(&mut self) {
        // Transition from Playing to Paused
        if let GameState::Playing { level, score, lives } = self {
            *self = GameState::Paused {
                level: *level,
                score: *score,
                lives: *lives,
            };
        }
    }

    fn resume(&mut self) {
        // Transition from Paused to Playing
        todo!()
    }

    fn add_score(&mut self, points: u32) {
        // Add points if in Playing state
        todo!()
    }

    fn lose_life(&mut self) {
        // Decrease lives, game over if 0
        todo!()
    }

    fn next_level(&mut self) {
        // Advance to next level, bonus points
        todo!()
    }

    fn is_game_over(&self) -> bool {
        matches!(self, GameState::GameOver { .. })
    }

    fn display(&self) {
        match self {
            GameState::Menu => println!("=== MAIN MENU ==="),
            GameState::Playing { level, score, lives } => {
                println!("Level: {} | Score: {} | Lives: {}", level, score, lives)
            }
            GameState::Paused { level, score, lives } => {
                println!("PAUSED - Level: {} | Score: {} | Lives: {}", level, score, lives)
            }
            GameState::GameOver { final_score } => {
                println!("GAME OVER - Final Score: {}", final_score)
            }
        }
    }
}

fn main() {
    let mut game = GameState::new();
    game.display();

    game.start_game();
    game.display();

    game.add_score(100);
    game.display();

    game.pause();
    game.display();

    game.resume();
    game.add_score(50);
    game.display();

    game.next_level();
    game.display();

    game.lose_life();
    game.lose_life();
    game.lose_life();
    game.display();

    assert!(game.is_game_over());
}
```

---

## Exercise 9: Shape Calculator

**Difficulty**: Medium
**Time**: 40 minutes

Create a shape enum with area calculations.

**Requirements**:
```rust
#[derive(Debug)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }

    fn perimeter(&self) -> f64 {
        // Implement perimeter calculation
        // For triangle, assume equilateral for simplicity
        todo!()
    }

    fn scale(&mut self, factor: f64) {
        // Scale the shape by factor
        todo!()
    }
}

fn total_area(shapes: &[Shape]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

fn main() {
    let shapes = vec![
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle { width: 10.0, height: 20.0 },
        Shape::Triangle { base: 6.0, height: 8.0 },
    ];

    for shape in &shapes {
        println!("{:?} has area: {:.2}", shape, shape.area());
    }

    println!("Total area: {:.2}", total_area(&shapes));
}
```

---

## Bonus Challenge: JSON-like Data Structure

**Difficulty**: Very Hard
**Time**: 90+ minutes

Create a JSON-like data structure using enums.

**Requirements**:
```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    fn get(&self, key: &str) -> Option<&JsonValue> {
        // Get value from object by key
        match self {
            JsonValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    fn get_index(&self, index: usize) -> Option<&JsonValue> {
        // Get value from array by index
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    fn as_bool(&self) -> Option<bool> {
        // Return Some(bool) if this is a Bool, None otherwise
        todo!()
    }

    fn as_number(&self) -> Option<f64> {
        todo!()
    }

    fn as_str(&self) -> Option<&str> {
        todo!()
    }

    fn to_pretty_string(&self, indent: usize) -> String {
        // Format as pretty JSON string
        todo!()
    }
}

fn main() {
    let mut user = HashMap::new();
    user.insert("name".to_string(), JsonValue::String("Alice".to_string()));
    user.insert("age".to_string(), JsonValue::Number(30.0));
    user.insert("active".to_string(), JsonValue::Bool(true));

    let json = JsonValue::Object(user);

    if let Some(name) = json.get("name") {
        println!("Name: {:?}", name);
    }

    assert_eq!(
        json.get("age").and_then(|v| v.as_number()),
        Some(30.0)
    );
}
```

---

## Check Your Understanding

Before moving to the next module, ensure you can:

- [ ] Define and use structs
- [ ] Create tuple structs and unit structs
- [ ] Implement methods and associated functions
- [ ] Define and use enums
- [ ] Work with Option<T> effectively
- [ ] Use pattern matching with match
- [ ] Destructure structs and enums in patterns
- [ ] Use if let and while let
- [ ] Understand when to use structs vs enums

---

## Common Mistakes to Avoid

1. **Forgetting to make struct mutable**: Must use `mut` to modify fields
2. **Incomplete match patterns**: Match must be exhaustive
3. **Unwrapping None**: Always handle the None case safely
4. **Confusing associated functions and methods**: `::new()` vs `.method()`
5. **Not using pattern matching**: Match is often clearer than if/else chains

---

## Additional Resources

- [Rust Book Chapter 5](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust Book Chapter 6](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [Pattern Syntax](https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html)
