# Exercises: Error Handling

## Exercise 1: Panic Basics

**Difficulty**: Easy
**Time**: 15 minutes

Understand when to use panic.

**Task 1**: Intentional panic
```rust
fn main() {
    // Different ways to panic
    panic!("This is a panic message");

    // Panic with formatted message
    let x = 5;
    panic!("x is {}, which is invalid", x);
}
```

**Task 2**: Panic in conditions
```rust
fn set_age(age: i32) {
    if age < 0 || age > 150 {
        panic!("Age {} is unrealistic", age);
    }
    println!("Age set to {}", age);
}

fn main() {
    set_age(25);   // OK
    set_age(-5);   // Panics!
}
```

**Task 3**: Array out of bounds
```rust
fn main() {
    let v = vec![1, 2, 3];

    // This panics:
    let element = v[99];

    // Better way:
    match v.get(99) {
        Some(element) => println!("Element: {}", element),
        None => println!("No element at index 99"),
    }
}
```

**Question**: When should you use panic vs return Result?

---

## Exercise 2: Result Basics

**Difficulty**: Easy
**Time**: 20 minutes

Master the Result type.

**Task 1**: Simple Result function
```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match divide(10.0, 0.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Task 2**: Result with custom error type
```rust
#[derive(Debug)]
enum DivisionError {
    DivideByZero,
    Overflow,
}

fn safe_divide(a: i32, b: i32) -> Result<i32, DivisionError> {
    if b == 0 {
        return Err(DivisionError::DivideByZero);
    }

    let result = a.checked_div(b);
    match result {
        Some(value) => Ok(value),
        None => Err(DivisionError::Overflow),
    }
}
```

---

## Exercise 3: The ? Operator

**Difficulty**: Medium
**Time**: 25 minutes

Master error propagation with `?`.

**Task 1**: Manual propagation vs ?
```rust
use std::fs::File;
use std::io::{self, Read};

// Manual propagation (verbose)
fn read_username_from_file_v1() -> Result<String, io::Error> {
    let f = File::open("username.txt");

    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut s = String::new();

    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

// Using ? operator (concise)
fn read_username_from_file_v2() -> Result<String, io::Error> {
    let mut f = File::open("username.txt")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

// Even more concise
fn read_username_from_file_v3() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("username.txt")?.read_to_string(&mut s)?;
    Ok(s)
}

// Most concise
fn read_username_from_file_v4() -> Result<String, io::Error> {
    std::fs::read_to_string("username.txt")
}
```

**Task 2**: Chain operations with ?
```rust
fn parse_and_double(s: &str) -> Result<i32, std::num::ParseIntError> {
    let n: i32 = s.parse()?;
    Ok(n * 2)
}

fn sum_from_strings(s1: &str, s2: &str) -> Result<i32, std::num::ParseIntError> {
    let n1: i32 = s1.parse()?;
    let n2: i32 = s2.parse()?;
    Ok(n1 + n2)
}

fn main() {
    match sum_from_strings("10", "20") {
        Ok(sum) => println!("Sum: {}", sum),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

---

## Exercise 4: Custom Error Types

**Difficulty**: Medium
**Time**: 35 minutes

Create your own error types.

**Task 1**: Simple custom error
```rust
use std::fmt;

#[derive(Debug, Clone)]
struct ValidationError {
    field: String,
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Validation error in {}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

fn validate_age(age: i32) -> Result<i32, ValidationError> {
    if age < 0 {
        Err(ValidationError {
            field: "age".to_string(),
            message: "Age cannot be negative".to_string(),
        })
    } else if age > 150 {
        Err(ValidationError {
            field: "age".to_string(),
            message: "Age is unrealistically high".to_string(),
        })
    } else {
        Ok(age)
    }
}

fn main() {
    match validate_age(-5) {
        Ok(age) => println!("Valid age: {}", age),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Task 2**: Error enum for multiple error types
```rust
#[derive(Debug)]
enum UserError {
    InvalidAge(i32),
    InvalidEmail(String),
    UsernameTooShort(usize),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserError::InvalidAge(age) => {
                write!(f, "Invalid age: {}", age)
            }
            UserError::InvalidEmail(email) => {
                write!(f, "Invalid email: {}", email)
            }
            UserError::UsernameTooShort(len) => {
                write!(f, "Username too short: {} characters", len)
            }
        }
    }
}

impl std::error::Error for UserError {}

fn validate_user(age: i32, email: &str, username: &str) -> Result<(), UserError> {
    if age < 0 || age > 150 {
        return Err(UserError::InvalidAge(age));
    }
    if !email.contains('@') {
        return Err(UserError::InvalidEmail(email.to_string()));
    }
    if username.len() < 3 {
        return Err(UserError::UsernameTooShort(username.len()));
    }
    Ok(())
}
```

---

## Exercise 5: Converting Between Error Types

**Difficulty**: Hard
**Time**: 40 minutes

Handle multiple error types in one function.

**Task 1**: Using Box<dyn Error>
```rust
use std::error::Error;
use std::fs::File;
use std::io::Read;

fn read_number_from_file(path: &str) -> Result<i32, Box<dyn Error>> {
    let mut file = File::open(path)?;  // io::Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;  // io::Error
    let number: i32 = contents.trim().parse()?;  // ParseIntError
    Ok(number)
}
```

**Task 2**: Custom error with From trait
```rust
use std::fmt;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum MyError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Io(e) => write!(f, "IO error: {}", e),
            MyError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for MyError {}

impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self {
        MyError::Io(error)
    }
}

impl From<ParseIntError> for MyError {
    fn from(error: ParseIntError) -> Self {
        MyError::Parse(error)
    }
}

fn read_number(path: &str) -> Result<i32, MyError> {
    let mut file = File::open(path)?;  // Automatically converts io::Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let number: i32 = contents.trim().parse()?;  // Automatically converts ParseIntError
    Ok(number)
}
```

---

## Exercise 6: Option to Result Conversion

**Difficulty**: Medium
**Time**: 20 minutes

Convert between Option and Result.

**Task 1**: ok_or and ok_or_else
```rust
fn get_user_id(username: &str) -> Option<u32> {
    // Simulate database lookup
    if username == "alice" {
        Some(1)
    } else {
        None
    }
}

fn get_user_id_result(username: &str) -> Result<u32, String> {
    get_user_id(username)
        .ok_or(format!("User '{}' not found", username))
}

fn get_user_id_lazy(username: &str) -> Result<u32, String> {
    get_user_id(username)
        .ok_or_else(|| format!("User '{}' not found", username))
}

fn main() {
    match get_user_id_result("alice") {
        Ok(id) => println!("User ID: {}", id),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Task 2**: transpose
```rust
fn parse_optional(s: Option<&str>) -> Result<Option<i32>, std::num::ParseIntError> {
    // If Some, parse and return Result
    // If None, return Ok(None)
    match s {
        Some(s) => s.parse().map(Some),
        None => Ok(None),
    }
}

// Using transpose (requires iterator)
fn parse_optional_v2(s: Option<&str>) -> Result<Option<i32>, std::num::ParseIntError> {
    s.map(str::parse).transpose()
}
```

---

## Exercise 7: Error Recovery

**Difficulty**: Medium
**Time**: 30 minutes

Implement error recovery strategies.

**Task 1**: Retry logic
```rust
use std::thread;
use std::time::Duration;

fn flaky_operation() -> Result<String, &'static str> {
    // Simulates an operation that might fail
    use rand::Rng;
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.7) {
        Err("Operation failed")
    } else {
        Ok("Success!".to_string())
    }
}

fn retry<F, T, E>(mut operation: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn main() {
    match retry(flaky_operation, 5) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Failed after retries: {}", e),
    }
}
```

**Task 2**: Fallback values
```rust
fn get_config_value(key: &str) -> Result<String, String> {
    // Simulate config lookup
    Err(format!("Key '{}' not found", key))
}

fn get_config_with_default(key: &str, default: &str) -> String {
    get_config_value(key).unwrap_or_else(|_| default.to_string())
}

fn main() {
    let timeout = get_config_with_default("timeout", "30");
    println!("Timeout: {}", timeout);
}
```

---

## Exercise 8: File Configuration Parser

**Difficulty**: Hard
**Time**: 60 minutes

Build a configuration file parser with comprehensive error handling.

**Requirements**:
```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::fmt;

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    InvalidFormat { line_num: usize, line: String },
    InvalidValue { key: String, value: String, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::InvalidFormat { line_num, line } => {
                write!(f, "Invalid format at line {}: {}", line_num, line)
            }
            ConfigError::InvalidValue { key, value, reason } => {
                write!(f, "Invalid value for '{}' = '{}': {}", key, value, reason)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::Io(error)
    }
}

struct Config {
    values: HashMap<String, String>,
}

impl Config {
    fn from_file(path: &str) -> Result<Self, ConfigError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut values = HashMap::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse "key = value" format
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::InvalidFormat {
                    line_num: line_num + 1,
                    line: line.to_string(),
                });
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    fn get_or(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    fn get_int(&self, key: &str) -> Result<i32, ConfigError> {
        let value = self.values.get(key).ok_or_else(|| {
            ConfigError::InvalidValue {
                key: key.to_string(),
                value: String::new(),
                reason: "Key not found".to_string(),
            }
        })?;

        value.parse().map_err(|_| ConfigError::InvalidValue {
            key: key.to_string(),
            value: value.clone(),
            reason: "Not a valid integer".to_string(),
        })
    }

    fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self.values.get(key).ok_or_else(|| {
            ConfigError::InvalidValue {
                key: key.to_string(),
                value: String::new(),
                reason: "Key not found".to_string(),
            }
        })?;

        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ConfigError::InvalidValue {
                key: key.to_string(),
                value: value.clone(),
                reason: "Not a valid boolean".to_string(),
            }),
        }
    }
}

fn main() -> Result<(), ConfigError> {
    // Create a test config file first:
    // port = 8080
    // host = localhost
    // debug = true
    // # This is a comment

    let config = Config::from_file("config.txt")?;

    let port = config.get_int("port")?;
    let host = config.get_or("host", "0.0.0.0");
    let debug = config.get_bool("debug")?;

    println!("Server starting on {}:{}", host, port);
    println!("Debug mode: {}", debug);

    Ok(())
}
```

---

## Exercise 9: Using thiserror Crate

**Difficulty**: Medium
**Time**: 25 minutes

Use the `thiserror` crate for ergonomic error types.

**Setup**:
```toml
[dependencies]
thiserror = "1.0"
```

**Task**:
```rust
use thiserror::Error;
use std::io;
use std::num::ParseIntError;

#[derive(Error, Debug)]
enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseIntError),

    #[error("Invalid value: {value} (must be between {min} and {max})")]
    OutOfRange {
        value: i32,
        min: i32,
        max: i32,
    },

    #[error("Custom error: {0}")]
    Custom(String),
}

fn validate_percentage(value: i32) -> Result<i32, DataError> {
    if value < 0 || value > 100 {
        Err(DataError::OutOfRange {
            value,
            min: 0,
            max: 100,
        })
    } else {
        Ok(value)
    }
}

fn main() {
    match validate_percentage(150) {
        Ok(v) => println!("Valid: {}", v),
        Err(e) => println!("Error: {}", e),
    }
}
```

---

## Bonus Challenge: Result Combinators

**Difficulty**: Hard
**Time**: 45 minutes

Master Result combinators for functional error handling.

**Task**: Implement a user registration system
```rust
#[derive(Debug)]
struct User {
    username: String,
    email: String,
    age: u32,
}

fn validate_username(username: &str) -> Result<String, String> {
    if username.len() < 3 {
        Err("Username too short".to_string())
    } else if username.len() > 20 {
        Err("Username too long".to_string())
    } else {
        Ok(username.to_string())
    }
}

fn validate_email(email: &str) -> Result<String, String> {
    if email.contains('@') {
        Ok(email.to_string())
    } else {
        Err("Invalid email format".to_string())
    }
}

fn validate_age(age: u32) -> Result<u32, String> {
    if age < 13 {
        Err("Must be at least 13 years old".to_string())
    } else if age > 120 {
        Err("Age is unrealistic".to_string())
    } else {
        Ok(age)
    }
}

fn register_user(username: &str, email: &str, age: u32) -> Result<User, Vec<String>> {
    let username_result = validate_username(username);
    let email_result = validate_email(email);
    let age_result = validate_age(age);

    // Collect all errors
    let mut errors = Vec::new();

    if let Err(e) = &username_result {
        errors.push(e.clone());
    }
    if let Err(e) = &email_result {
        errors.push(e.clone());
    }
    if let Err(e) = &age_result {
        errors.push(e.clone());
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(User {
        username: username_result.unwrap(),
        email: email_result.unwrap(),
        age: age_result.unwrap(),
    })
}

fn main() {
    match register_user("ab", "invalid", 10) {
        Ok(user) => println!("User registered: {:?}", user),
        Err(errors) => {
            println!("Registration failed:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}
```

---

## Check Your Understanding

Before moving to the next module, ensure you can:

- [ ] Understand when to panic vs return Result
- [ ] Use the ? operator for error propagation
- [ ] Create custom error types
- [ ] Implement Display and Error traits for custom errors
- [ ] Convert between error types using From
- [ ] Handle multiple error types in one function
- [ ] Convert between Option and Result
- [ ] Use Result combinators (map, and_then, etc.)
- [ ] Implement error recovery strategies

---

## Common Mistakes to Avoid

1. **Overusing panic**: Use Result for recoverable errors
2. **Swallowing errors**: Always handle or propagate errors
3. **Using unwrap() in production**: Prefer match, unwrap_or, or ?
4. **Not providing context**: Include helpful error messages
5. **Ignoring error types**: Use appropriate error types

---

## Additional Resources

- [Rust Book Chapter 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror crate](https://docs.rs/thiserror/)
- [anyhow crate](https://docs.rs/anyhow/)
- [Error Handling in Rust](https://blog.burntsushi.net/rust-error-handling/)
