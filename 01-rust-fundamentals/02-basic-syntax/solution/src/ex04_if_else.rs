//! Exercise 4: Control Flow - If/Else.

/// Task 1. `if` is an expression, so the whole chain *is* the return value.
pub fn get_letter_grade(score: u32) -> char {
    if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else if score >= 60 {
        'D'
    } else {
        'F'
    }
}

/// Task 2. Both arms must have the same type because `number` has exactly
/// one type, fixed at compile time. `if condition { 5 } else { "six" }` is
/// error[E0308]: `if` and `else` have incompatible types.
#[allow(clippy::let_and_return)] // the `let` is what the exercise is about
pub fn if_in_let(condition: bool) -> i32 {
    let number = if condition { 5 } else { 6 };
    number
}

/// Task 3. Divisible by 4, except centuries, except every 400 years.
pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn run() {
    for score in [95, 85, 75, 65, 30] {
        println!("{score} -> {}", get_letter_grade(score));
    }
    println!(
        "if_in_let(true) = {}, if_in_let(false) = {}",
        if_in_let(true),
        if_in_let(false)
    );
    for year in [1900, 2000, 2023, 2024] {
        println!("{year} leap? {}", is_leap_year(year));
    }
}
