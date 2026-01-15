# Exercises: Basic Syntax

## Exercise 1: Variables and Mutability

**Difficulty**: Easy
**Time**: 15 minutes

Understand the difference between mutable and immutable variables.

**Tasks**:
1. Create immutable and mutable variables
2. Demonstrate shadowing
3. Show type annotations

**Code to complete**:
```rust
fn main() {
    // Create an immutable variable x with value 5

    // Try to change x (this should error)
    // x = 6;

    // Create a mutable variable y with value 10

    // Change y to 20

    // Shadow x with a new value of different type

    // Create a variable with explicit type annotation
}
```

**Questions**:
- What's the difference between shadowing and mutability?
- When would you use const vs let?
- Can you shadow a mutable variable with an immutable one?

---

## Exercise 2: Data Types

**Difficulty**: Easy
**Time**: 20 minutes

Work with Rust's scalar and compound types.

**Tasks**:
Create a program that demonstrates:
1. All integer types (i8, i16, i32, i64, i128, isize)
2. Floating point types (f32, f64)
3. Boolean type
4. Character type (including Unicode)
5. Tuple type
6. Array type

**Example**:
```rust
fn main() {
    // Integer types
    let small: i8 = 127;
    let big: i64 = 9_223_372_036_854_775_807;

    // Your code here...

    // Print sizes
    println!("i8 size: {} bytes", std::mem::size_of::<i8>());

    // Create a tuple with mixed types

    // Create an array of 5 integers

    // Access array elements
}
```

**Bonus**: Demonstrate integer overflow in debug vs release mode

---

## Exercise 3: Functions

**Difficulty**: Medium
**Time**: 30 minutes

Master function definitions, parameters, and return values.

**Requirements**:
Create functions for the following:

1. `greet(name: &str)` - Prints a greeting
2. `add(a: i32, b: i32) -> i32` - Returns sum
3. `is_even(n: i32) -> bool` - Returns true if even
4. `celsius_to_fahrenheit(c: f64) -> f64` - Converts temperature
5. `swap(a: i32, b: i32) -> (i32, i32)` - Returns swapped tuple

**Test your functions**:
```rust
fn main() {
    greet("Alice");
    assert_eq!(add(2, 3), 5);
    assert!(is_even(4));
    assert_eq!(celsius_to_fahrenheit(0.0), 32.0);

    let (x, y) = swap(1, 2);
    assert_eq!(x, 2);
    assert_eq!(y, 1);
}
```

**Bonus**: Create a function that takes another function as a parameter

---

## Exercise 4: Control Flow - If/Else

**Difficulty**: Easy
**Time**: 25 minutes

Practice conditional statements.

**Task 1**: Grade Calculator
```rust
fn get_letter_grade(score: u32) -> char {
    // Return letter grade based on score
    // 90-100: A
    // 80-89: B
    // 70-79: C
    // 60-69: D
    // Below 60: F
}
```

**Task 2**: Using if in let statement
```rust
fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 };
    // Explain why both arms must return the same type
}
```

**Task 3**: Leap Year Checker
```rust
fn is_leap_year(year: u32) -> bool {
    // A year is a leap year if:
    // - Divisible by 4
    // - NOT divisible by 100, UNLESS also divisible by 400
}
```

---

## Exercise 5: Loops

**Difficulty**: Medium
**Time**: 35 minutes

Master all loop types in Rust.

**Task 1**: Loop with break and return value
```rust
fn find_first_multiple_of_7() -> i32 {
    let mut counter = 1;
    loop {
        // Return the first number divisible by 7
    }
}
```

**Task 2**: While loop - Countdown
```rust
fn countdown(from: i32) {
    // Print countdown from 'from' to 1, then "Liftoff!"
}
```

**Task 3**: For loop - Fibonacci sequence
```rust
fn print_fibonacci(n: u32) {
    // Print first n numbers of Fibonacci sequence
    // 0, 1, 1, 2, 3, 5, 8, 13, ...
}
```

**Task 4**: Loop labels
```rust
fn nested_loop_example() {
    // Use loop labels to break from outer loop
    'outer: for i in 0..5 {
        for j in 0..5 {
            if i * j > 10 {
                // Break from outer loop
            }
        }
    }
}
```

---

## Exercise 6: Pattern Matching with Match

**Difficulty**: Medium
**Time**: 30 minutes

Master the match expression.

**Task 1**: Coin value
```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u32 {
    // Use match to return the value
}
```

**Task 2**: Number classifier
```rust
fn classify_number(n: i32) -> &'static str {
    match n {
        // Negative numbers
        // Zero
        // Positive numbers 1-10
        // Everything else
    }
}
```

**Task 3**: Match with guards
```rust
fn describe_number(n: i32) -> String {
    match n {
        x if x < 0 => format!("{} is negative", x),
        // Add more cases with guards
    }
}
```

**Task 4**: Exhaustive matching
```rust
fn handle_input(input: Option<i32>) {
    match input {
        // Handle Some and None cases
    }
}
```

---

## Exercise 7: Calculator Program

**Difficulty**: Medium
**Time**: 45 minutes

Build a command-line calculator combining all concepts.

**Requirements**:
1. Support operations: +, -, *, /
2. Take input in format: "5 + 3"
3. Handle division by zero
4. Support decimal numbers
5. Continue until user types "quit"

**Structure**:
```rust
fn parse_operation(input: &str) -> Option<(f64, char, f64)> {
    // Parse "5 + 3" into (5.0, '+', 3.0)
}

fn calculate(a: f64, op: char, b: f64) -> Option<f64> {
    // Perform calculation, return None for division by zero
}

fn main() {
    loop {
        // Read input
        // Parse operation
        // Calculate result
        // Print result or error
        // Break on "quit"
    }
}
```

**Expected Output**:
```
Calculator (type 'quit' to exit)
> 5 + 3
Result: 8
> 10 / 2
Result: 5
> 10 / 0
Error: Division by zero
> quit
Goodbye!
```

---

## Exercise 8: FizzBuzz

**Difficulty**: Easy
**Time**: 20 minutes

The classic FizzBuzz problem.

**Requirements**:
- Print numbers from 1 to 100
- For multiples of 3, print "Fizz"
- For multiples of 5, print "Buzz"
- For multiples of both 3 and 5, print "FizzBuzz"
- Otherwise, print the number

**Implement three versions**:
1. Using if/else
2. Using match
3. Using match with guards (most elegant)

---

## Exercise 9: Temperature Converter

**Difficulty**: Easy
**Time**: 25 minutes

Create a bidirectional temperature converter.

**Requirements**:
```rust
fn fahrenheit_to_celsius(f: f64) -> f64 {
    // (F - 32) * 5/9
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    // C * 9/5 + 32
}

fn main() {
    // Prompt user for:
    // 1. Temperature value
    // 2. Unit (C or F)
    // 3. Convert and display result
}
```

**Test cases**:
- 0°C = 32°F
- 100°C = 212°F
- 32°F = 0°C
- -40°F = -40°C (same in both!)

---

## Bonus Challenge: Prime Number Generator

**Difficulty**: Hard
**Time**: 60 minutes

Generate prime numbers efficiently.

**Requirements**:
1. Implement `is_prime(n: u64) -> bool`
2. Implement `nth_prime(n: usize) -> u64` - Return the nth prime
3. Implement `primes_up_to(limit: u64) -> Vec<u64>` - All primes up to limit
4. Optimize using Sieve of Eratosthenes

**Tests**:
```rust
assert!(is_prime(2));
assert!(is_prime(17));
assert!(!is_prime(1));
assert!(!is_prime(4));

assert_eq!(nth_prime(1), 2);
assert_eq!(nth_prime(10), 29);

assert_eq!(primes_up_to(10), vec![2, 3, 5, 7]);
```

---

## Check Your Understanding

Before moving to the next module, ensure you can:

- [ ] Declare and use variables (mutable and immutable)
- [ ] Understand and use all primitive data types
- [ ] Write functions with parameters and return values
- [ ] Use if/else expressions
- [ ] Work with all loop types (loop, while, for)
- [ ] Pattern match with match expressions
- [ ] Understand expressions vs statements
- [ ] Handle user input and parse strings

---

## Common Mistakes to Avoid

1. **Forgetting semicolons**: Last line in function without ; returns value
2. **Type mismatches in if/else**: Both arms must return same type
3. **Infinite loops**: Ensure loop has exit condition
4. **Integer overflow**: Use checked arithmetic or appropriate types
5. **Shadowing confusion**: Shadowing creates new variable

---

## Additional Resources

- [Rust Book Chapter 3](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html)
- [Rust by Example - Flow Control](https://doc.rust-lang.org/rust-by-example/flow_control.html)
