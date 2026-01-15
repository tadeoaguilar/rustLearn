# Phase 1: Rust Fundamentals

Master the core concepts that make Rust unique and powerful.

## Overview

This phase introduces you to Rust's fundamental concepts, including its unique ownership system, type system, and error handling patterns. These concepts are the foundation for everything you'll learn in Rust.

**Duration**: 4-6 weeks
**Difficulty**: Beginner
**Prerequisites**: Basic programming knowledge

## Modules

### [01-getting-started](01-getting-started/)

**Learning Objectives**:
- Install Rust using rustup
- Understand cargo and the Rust toolchain
- Write and run your first Rust program
- Navigate Rust documentation
- Use basic debugging tools

**Key Topics**:
- Installing rustup and managing toolchains
- Cargo basics: new, build, run, test
- Project structure conventions
- Using rust-analyzer for IDE support
- Reading compiler error messages

**Exercises**:
1. Install Rust and verify installation
2. Create a "Hello, World!" program
3. Build a simple calculator CLI
4. Explore the standard library docs

---

### [02-basic-syntax](02-basic-syntax/)

**Learning Objectives**:
- Understand Rust's type system fundamentals
- Work with variables, mutability, and shadowing
- Use control flow structures
- Define and call functions
- Understand expression vs statement

**Key Topics**:
- Scalar types: integers, floats, bool, char
- Compound types: tuples, arrays
- Variables and mutability (`let` vs `let mut`)
- Shadowing and type annotations
- Functions, parameters, return values
- Control flow: if, loop, while, for
- Match expressions
- Expressions vs statements

**Exercises**:
1. Temperature converter (Celsius/Fahrenheit)
2. Fibonacci sequence generator
3. FizzBuzz implementation
4. Simple grade calculator

---

### [03-ownership-borrowing](03-ownership-borrowing/)

**Learning Objectives**:
- Understand Rust's ownership model
- Master borrowing and references
- Work with mutable and immutable references
- Understand the borrowing rules
- Grasp the concept of move semantics

**Key Topics**:
- Ownership rules (each value has one owner)
- Move semantics (transfer of ownership)
- Clone and Copy traits
- References and borrowing (&T)
- Mutable references (&mut T)
- Borrowing rules (one mutable OR many immutable)
- Lifetimes introduction
- String vs &str

**Exercises**:
1. Implement a function that calculates string length without taking ownership
2. Build a simple text buffer with mutable operations
3. Create a reference counter manually
4. Debug ownership compilation errors

**Resources**:
- [Rust Book Chapter 4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Visualizing Memory Layout](https://fasterthanli.me/articles/i-am-a-java-csharp-c-or-cplusplus-dev-time-to-do-some-rust)

---

### [04-structs-enums](04-structs-enums/)

**Learning Objectives**:
- Define and use custom types
- Implement methods on structs
- Master pattern matching with enums
- Use Option and Result effectively
- Understand algebraic data types

**Key Topics**:
- Struct definition and instantiation
- Tuple structs and unit structs
- Implementing methods with `impl` blocks
- Associated functions (static methods)
- Enums and variants
- Pattern matching with `match`
- The Option<T> type (Some/None)
- The Result<T, E> type (Ok/Err)
- If let and while let syntax

**Exercises**:
1. Create a User struct with methods
2. Build a simple Shape enum with area calculations
3. Implement a basic calculator using Result for error handling
4. Create a simple state machine with enums

---

### [05-error-handling](05-error-handling/)

**Learning Objectives**:
- Master Rust's error handling philosophy
- Use Result and Option effectively
- Propagate errors with the `?` operator
- Create custom error types
- Understand panic vs recoverable errors

**Key Topics**:
- Panic and unrecoverable errors
- Result<T, E> for recoverable errors
- Error propagation with `?` operator
- Creating custom error types
- The `thiserror` and `anyhow` crates
- Error handling best practices
- Converting between error types

**Exercises**:
1. Build a file reader with proper error handling
2. Create a custom error type for a calculator
3. Implement error propagation in a multi-function program
4. Parse user input with comprehensive error messages

**Resources**:
- [Rust Book Chapter 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror crate](https://docs.rs/thiserror/)
- [anyhow crate](https://docs.rs/anyhow/)

---

## Phase Completion Checklist

- [ ] Can write and run Rust programs confidently
- [ ] Understand ownership, borrowing, and lifetimes
- [ ] Can define and use custom types (structs and enums)
- [ ] Handle errors properly with Result and Option
- [ ] Comfortable reading Rust compiler error messages
- [ ] Completed at least 3 small projects

## Projects

Build these projects to solidify your understanding:

1. **CLI Todo App**: CRUD operations, file persistence, error handling
2. **Text Analyzer**: Count words, lines, characters; find patterns
3. **Number Guessing Game**: User input, random numbers, loops, match

## Next Steps

After completing this phase, you should:
- Move on to [02-intermediate-rust](../02-intermediate-rust/)
- Join the Rust community forums
- Start solving problems on Exercism or LeetCode in Rust
- Read other people's Rust code on GitHub

## Additional Resources

- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Small exercises
- [Exercism Rust Track](https://exercism.org/tracks/rust)
