# Exercises: Getting Started

## Exercise 1: Hello, Rust!

**Difficulty**: Easy
**Time**: 10 minutes

Create a program that prints a personalized greeting.

**Requirements**:
1. Create a new Rust project called `hello_rust`
2. Modify the program to ask for the user's name
3. Print a greeting with their name
4. Print the Rust version being used

**Expected Output**:
```
What's your name? John
Hello, John! Welcome to Rust!
You're using Rust version: 1.75.0
```

**Hints**:
- Use `cargo new hello_rust`
- Use `std::io::stdin()` for input
- Use `rustc --version` information

---

## Exercise 2: Cargo Basics

**Difficulty**: Easy
**Time**: 15 minutes

Practice using Cargo commands and understand project structure.

**Tasks**:
1. Create a new library crate called `my_math`
2. Add a function `add(a: i32, b: i32) -> i32`
3. Create a binary that uses this library
4. Run tests with `cargo test`
5. Build in release mode
6. Check the project with `cargo check`

**Questions to Answer**:
- What's the difference between `cargo build` and `cargo build --release`?
- Where are the compiled binaries stored?
- What files does Cargo generate?

---

## Exercise 3: Documentation Explorer

**Difficulty**: Easy
**Time**: 20 minutes

Learn to navigate Rust documentation effectively.

**Tasks**:
1. Generate documentation for a project with `cargo doc --open`
2. Find the documentation for `Vec<T>` in the standard library
3. Read about the `String` type and its methods
4. Find 3 different ways to create a String

**Write a summary**:
- What's the difference between `String` and `&str`?
- How do you find documentation for a specific method?
- What does the `std::prelude` module contain?

---

## Exercise 4: First Real Program - Number Guessing Game

**Difficulty**: Medium
**Time**: 45 minutes

Build the classic number guessing game.

**Requirements**:
1. Generate a random number between 1 and 100
2. Ask the user to guess the number
3. Tell them if their guess is too high or too low
4. Count the number of guesses
5. Congratulate them when they guess correctly

**Dependencies needed**:
```toml
[dependencies]
rand = "0.8"
```

**Expected Output**:
```
Guess the number!
Please input your guess: 50
Too small!
Please input your guess: 75
Too big!
Please input your guess: 62
You got it in 3 guesses!
```

**Hints**:
- Use `rand::thread_rng()` and `gen_range(1..=100)`
- Use a loop to keep asking for guesses
- Use `.trim()` on input strings
- Parse the input with `.parse::<u32>()`

---

## Exercise 5: Project Setup Best Practices

**Difficulty**: Easy
**Time**: 20 minutes

Set up a professional Rust project with proper tooling.

**Tasks**:
1. Create a new project called `rust_tools`
2. Add a `.gitignore` file for Rust
3. Create a `README.md` with project description
4. Set up `rustfmt` configuration
5. Set up `clippy` lints in `Cargo.toml`

**Add to Cargo.toml**:
```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
enum_glob_use = "deny"
```

**Test your setup**:
```bash
cargo fmt
cargo clippy
cargo test
```

---

## Exercise 6: Debugging Practice

**Difficulty**: Medium
**Time**: 30 minutes

Practice debugging Rust programs.

**Tasks**:
1. Create a program with intentional bugs:
   - Unused variable
   - Wrong type conversion
   - Missing semicolon in return
2. Use compiler error messages to fix them
3. Add `println!` debugging statements
4. Use `dbg!()` macro to inspect values

**Example buggy code**:
```rust
fn main() {
    let x = 5;
    let y = "10";
    let result = x + y;
    println!("Result: {}", result);
}
```

**Fix the code and document**:
- What was each error?
- How did the compiler help you?
- What's the difference between `println!()` and `dbg!()`?

---

## Exercise 7: Environment Setup

**Difficulty**: Easy
**Time**: 15 minutes

Configure your development environment optimally.

**Tasks**:
1. Install `rust-analyzer` for your IDE/editor
2. Configure auto-formatting on save
3. Enable clippy hints in your editor
4. Set up keyboard shortcuts for:
   - Running tests
   - Running the current file
   - Building the project

**Verify**:
- Type inference works (hover over variables)
- Auto-completion works
- Jump to definition works
- Error highlighting appears inline

---

## Bonus Challenge: Cargo Extension

**Difficulty**: Hard
**Time**: 60 minutes

Explore Cargo extensions and create a build script.

**Tasks**:
1. Install `cargo-watch`: `cargo install cargo-watch`
2. Install `cargo-edit`: `cargo install cargo-edit`
3. Create a build script (`build.rs`) that prints the build timestamp
4. Use `cargo add` to add a dependency
5. Use `cargo watch -x test` to auto-run tests

**Build script example**:
```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=BUILD_TIME={}",
             chrono::Utc::now().to_rfc3339());
}
```

---

## Check Your Understanding

Before moving to the next module, ensure you can:

- [ ] Create new Rust projects with cargo
- [ ] Understand the project structure (src/, Cargo.toml, target/)
- [ ] Run, build, test, and check projects
- [ ] Read and understand compiler errors
- [ ] Navigate Rust documentation
- [ ] Use basic debugging techniques
- [ ] Configure your development environment

---

## Additional Resources

- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Playground](https://play.rust-lang.org/) - Test code in browser
- [rust-analyzer Manual](https://rust-analyzer.github.io/manual.html)
