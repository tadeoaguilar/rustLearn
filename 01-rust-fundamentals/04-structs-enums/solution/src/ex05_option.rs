//! Exercise 5: Option<T>.
//!
//! Rust has no null. A value that might be missing has type `Option<T>`, and
//! the compiler won't let you use the `T` inside until you have checked.

/// Task 1.
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

pub fn expensive_computation() -> i32 {
    42
}

/// Task 2: the Option methods from the exercise, each result returned so the
/// tests can check them. Read the comments, not just the code.
// Clippy flags unwrapping a literal Some/None as pointless -- it is, outside
// of a demonstration like this one.
#[allow(clippy::unnecessary_literal_unwrap)]
pub fn option_methods() -> [Option<i32>; 6] {
    let some_number = Some(5);
    let no_number: Option<i32> = None;

    // unwrap() panics on None. Fine in tests and examples; in real code
    // prefer one of the alternatives below, or `expect("why this can't fail")`.
    let a = some_number.unwrap();

    // unwrap_or(default): the default is computed even when not needed.
    let b = no_number.unwrap_or(0);

    // unwrap_or_else(closure): the closure only runs on None. Use this when
    // the default is expensive.
    let c = no_number.unwrap_or_else(expensive_computation);

    // map: transform the inside, keep the Option wrapper.
    let doubled = some_number.map(|n| n * 2);

    // and_then: the closure itself returns an Option. (map would give you
    // Option<Option<i32>>.)
    let chained = some_number.and_then(|n| if n > 3 { Some(n * 2) } else { None });

    // filter: keep Some only if the predicate holds.
    let filtered = some_number.filter(|n| n % 2 == 0);

    [Some(a), Some(b), Some(c), doubled, chained, filtered]
}

/// Task 3. Iterator adaptors already return Option: `find` and `position`.
pub fn find_first_even(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().find(|n| n % 2 == 0)
}

pub fn find_position<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    slice.iter().position(|item| item == target)
}

/// The same as `find_position`, written with a plain loop, for comparison.
pub fn find_position_loop<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    for (i, item) in slice.iter().enumerate() {
        if item == target {
            return Some(i);
        }
    }
    None
}

pub fn run() {
    match divide(10.0, 2.0) {
        Some(result) => println!("Result: {result}"),
        None => println!("Cannot divide by zero"),
    }
    println!("divide(1, 0) = {:?}", divide(1.0, 0.0));
    println!("option_methods() = {:?}", option_methods());

    let numbers = vec![1, 3, 5, 8, 9, 10];
    println!("first even = {:?}", find_first_even(&numbers));
    println!("position of 5 = {:?}", find_position(&numbers, &5));
    println!("position of 100 = {:?}", find_position(&numbers, &100));
}
