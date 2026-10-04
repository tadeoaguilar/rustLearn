//! Bonus: an attribute macro. Write `timed` in ../derive/src/lib.rs.
//! These functions already work; #[timed] must not change what they return.

use crate::timed;

#[timed]
pub fn slow_sum(n: u64) -> u64 {
    (0..n).sum()
}

#[timed("parsing")]
pub fn parse_positive(s: &str) -> Result<i32, String> {
    let n: i32 = s.trim().parse().map_err(|e| format!("{e}"))?;
    if n <= 0 {
        return Err(format!("{n} is not positive"));
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
    println!("{}", greet("Ferris".to_string()));
    println!("(with #[timed] implemented, timings appear on stderr)");
}
