//! Exercise 8: FizzBuzz -- three versions that must agree with each other.

/// Version 1: if/else. The `% 15` check has to come first.
pub fn fizzbuzz_if(n: u32) -> String {
    if n % 15 == 0 {
        "FizzBuzz".to_string()
    } else if n % 3 == 0 {
        "Fizz".to_string()
    } else if n % 5 == 0 {
        "Buzz".to_string()
    } else {
        n.to_string()
    }
}

/// Version 2: match on a tuple of remainders. Every combination is visible
/// at a glance, and `_` means "don't care".
pub fn fizzbuzz_match(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}

/// Version 3: match with guards.
pub fn fizzbuzz_guard(n: u32) -> String {
    match n {
        n if n % 15 == 0 => "FizzBuzz".to_string(),
        n if n % 3 == 0 => "Fizz".to_string(),
        n if n % 5 == 0 => "Buzz".to_string(),
        n => n.to_string(),
    }
}

/// 1 to `limit`, using any of the three versions.
pub fn fizzbuzz_up_to(limit: u32, f: fn(u32) -> String) -> Vec<String> {
    (1..=limit).map(f).collect()
}

pub fn run() {
    println!("{}", fizzbuzz_up_to(100, fizzbuzz_match).join(" "));
}
