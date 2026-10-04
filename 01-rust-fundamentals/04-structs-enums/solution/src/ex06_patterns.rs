//! Exercise 6: Pattern Matching.

/// Task 1: `|` for alternatives, `..=` for inclusive ranges.
pub fn describe_number(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1 | 2 => "one or two",
        3..=9 => "three through nine",
        10 => "ten",
        _ => "something else",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Task 2: destructuring with literals inside the pattern.
pub fn locate(point: Point) -> String {
    match point {
        Point { x: 0, y: 0 } => "At the origin".to_string(),
        Point { x: 0, y } => format!("On y-axis at {y}"),
        Point { x, y: 0 } => format!("On x-axis at {x}"),
        Point { x, y } => format!("At ({x}, {y})"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Temperature {
    Celsius(i32),
    Fahrenheit(i32),
}

/// Task 3: guards. Note the `_` arm is required: the compiler does not reason
/// about guard conditions, so without it the match is "non-exhaustive" even if
/// you covered every number.
pub fn describe_temperature(temp: Temperature) -> &'static str {
    match temp {
        Temperature::Celsius(t) if t < 0 => "freezing in Celsius",
        Temperature::Celsius(t) if t > 30 => "hot in Celsius",
        Temperature::Fahrenheit(t) if t < 32 => "freezing in Fahrenheit",
        Temperature::Fahrenheit(t) if t > 86 => "hot in Fahrenheit",
        _ => "moderate temperature",
    }
}

/// Task 4: `if let` for "I only care about one pattern".
pub fn is_three(value: Option<i32>) -> bool {
    if let Some(3) = value {
        return true;
    }
    false
    // Even shorter for a yes/no question: matches!(value, Some(3))
}

/// Task 4: `while let` loops as long as the pattern matches -- here, until
/// `pop` returns None.
pub fn drain_stack(mut stack: Vec<i32>) -> Vec<i32> {
    let mut popped = Vec::new();
    while let Some(top) = stack.pop() {
        popped.push(top);
    }
    popped
}

/// Bonus pattern: `let else` (Rust 1.65+) -- bind the pattern, or run the
/// `else` block, which must leave the function (return, break, panic...).
/// Useful when the early exit isn't just "return None" -- for that, `?` is
/// shorter, as in `parse_pair` below.
pub fn describe_pair(s: &str) -> String {
    let Some((a, b)) = s.split_once(',') else {
        return format!("{s:?} has no comma");
    };
    format!("{} and {}", a.trim(), b.trim())
}

/// `?` on Option: any None returns None from the whole function.
pub fn parse_pair(s: &str) -> Option<(i32, i32)> {
    let (a, b) = s.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

pub fn run() {
    for n in [0, 2, 7, 10, 99] {
        println!("{n}: {}", describe_number(n));
    }
    println!("{}", locate(Point { x: 0, y: 7 }));
    println!("{}", locate(Point { x: 3, y: 0 }));
    println!("{}", describe_temperature(Temperature::Celsius(-5)));
    println!("{}", describe_temperature(Temperature::Fahrenheit(100)));
    println!("is_three(Some(3)) = {}", is_three(Some(3)));
    println!("drain_stack([1, 2, 3]) = {:?}", drain_stack(vec![1, 2, 3]));
    println!("{}", describe_pair("salt, pepper"));
    println!("{}", describe_pair("salt"));
    println!("parse_pair(\"3, 4\") = {:?}", parse_pair("3, 4"));
}
