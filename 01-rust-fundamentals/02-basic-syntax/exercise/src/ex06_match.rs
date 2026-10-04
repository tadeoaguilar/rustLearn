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
    todo!("Exercise 6")
}

/// Task 2. Range patterns. Arms are tried top to bottom.
pub fn classify_number(n: i32) -> &'static str {
    todo!("Exercise 6")
}

/// Task 3. Guards (`if ...`) handle conditions a pattern cannot express.
pub fn describe_number(n: i32) -> String {
    todo!("Exercise 6")
}

/// Task 4. Both `Some` and `None` must be handled; drop either arm and the
/// compiler says error[E0004]: non-exhaustive patterns.
pub fn handle_input(input: Option<i32>) -> String {
    todo!("Exercise 6")
}

pub fn run() {
    todo!("Exercise 6")
}
