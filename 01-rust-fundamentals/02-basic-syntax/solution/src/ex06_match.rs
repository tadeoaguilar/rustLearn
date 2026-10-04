//! Exercise 6: Pattern Matching with Match.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

/// Task 1. Add a variant to `Coin` and this stops compiling until you handle
/// it -- that exhaustiveness check is match's superpower.
pub fn value_in_cents(coin: Coin) -> u32 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}

/// Task 2. Range patterns. Arms are tried top to bottom.
pub fn classify_number(n: i32) -> &'static str {
    match n {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        1..=10 => "small positive",
        _ => "large positive",
    }
}

/// Task 3. Guards (`if ...`) handle conditions a pattern cannot express.
pub fn describe_number(n: i32) -> String {
    match n {
        x if x < 0 => format!("{x} is negative"),
        0 => "0 is zero".to_string(),
        x if x % 2 == 0 => format!("{x} is positive and even"),
        x => format!("{x} is positive and odd"),
    }
}

/// Task 4. Both `Some` and `None` must be handled; drop either arm and the
/// compiler says error[E0004]: non-exhaustive patterns.
pub fn handle_input(input: Option<i32>) -> String {
    match input {
        Some(n) => format!("Got {n}"),
        None => "Got nothing".to_string(),
    }
}

pub fn run() {
    for coin in [Coin::Penny, Coin::Nickel, Coin::Dime, Coin::Quarter] {
        println!("{coin:?} = {} cents", value_in_cents(coin));
    }
    for n in [-5, 0, 7, 42] {
        println!("{n}: {} / {}", classify_number(n), describe_number(n));
    }
    println!("{} / {}", handle_input(Some(3)), handle_input(None));
}
