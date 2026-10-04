//! Exercise 1: Basic Async Functions.

use std::num::ParseIntError;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, sleep};

pub async fn say_hello() -> &'static str {
    "Hello"
}

/// `tokio::time::sleep`, never `std::thread::sleep`: the std version blocks
/// the whole worker thread, and every other task scheduled on it stalls.
pub async fn say_world() -> &'static str {
    sleep(Duration::from_millis(100)).await;
    "World"
}

/// Task 1: async functions that return values.
pub async fn add_async(a: i32, b: i32) -> i32 {
    a + b
}

/// Task 2: chaining -- each `.await` finishes before the next line runs.
pub async fn hello_world() -> String {
    let hello = say_hello().await;
    let world = say_world().await;
    format!("{hello} {world}")
}

/// Task 3: errors. `?` works in async fns exactly as in sync ones.
pub async fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.trim().parse()
}

pub async fn sum_strings(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x = parse_number(a).await?;
    let y = parse_number(b).await?;
    Ok(add_async(x, y).await)
}

/// Futures are lazy. Creating one runs *none* of its body; only awaiting
/// does. Returns (ran before await, ran after await) -- expect (false, true).
pub async fn futures_are_lazy() -> (bool, bool) {
    let ran = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&ran);
    let fut = async move {
        flag.store(true, Ordering::SeqCst);
    };
    let before = ran.load(Ordering::SeqCst);
    fut.await;
    (before, ran.load(Ordering::SeqCst))
}

pub async fn run() {
    println!("{}", hello_world().await);
    println!("add_async(2, 3) = {}", add_async(2, 3).await);
    println!(
        "sum_strings(\"10\", \"32\") = {:?}",
        sum_strings("10", "32").await
    );
    println!(
        "sum_strings(\"10\", \"x\")  = {:?}",
        sum_strings("10", "x").await.map_err(|e| e.to_string())
    );
    println!(
        "(ran before await, ran after await) = {:?}",
        futures_are_lazy().await
    );
}
