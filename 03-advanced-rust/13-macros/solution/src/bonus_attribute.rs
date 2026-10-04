//! Bonus: an attribute macro -- see `derive/src/lib.rs` for `#[timed]`.

use crate::timed;

#[timed]
pub fn slow_sum(n: u64) -> u64 {
    (0..n).sum()
}

#[timed("parsing")]
pub fn parse_positive(s: &str) -> Result<i32, String> {
    let n: i32 = s.trim().parse().map_err(|e| format!("{e}"))?; // `?` still returns from the fn
    if n <= 0 {
        return Err(format!("{n} is not positive")); // so does `return`
    }
    Ok(n)
}

#[timed]
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

pub fn run() {
    println!("slow_sum(1_000_000) = {}", slow_sum(1_000_000));
    println!("parse_positive(\"42\") = {:?}", parse_positive("42"));
    println!("parse_positive(\"-1\") = {:?}", parse_positive("-1"));
    println!("parse_positive(\"x\")  = {:?}", parse_positive("x"));
    println!("{}", greet("Ferris".to_string()));
    println!("(the [timed] lines went to stderr)");
}
