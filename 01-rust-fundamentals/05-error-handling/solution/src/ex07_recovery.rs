//! Exercise 7: Error Recovery.

use rand::Rng;
use std::thread;
use std::time::Duration;

/// Fails 70% of the time.
pub fn flaky_operation() -> Result<String, &'static str> {
    if rand::thread_rng().gen_bool(0.7) {
        Err("Operation failed")
    } else {
        Ok("Success!".to_string())
    }
}

/// Task 1 as in the exercise. Note that `max_retries` is really the maximum
/// number of *attempts*: with 5 the operation runs at most 5 times.
pub fn retry<F, T, E>(operation: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    retry_with_delay(operation, max_retries, Duration::from_millis(100))
}

/// The same, with the delay as a parameter so tests don't have to sleep.
/// `FnMut` because the operation may change state between calls (a counter,
/// a connection) -- `Fn` would be too strict, `FnOnce` can't be called twice.
pub fn retry_with_delay<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    assert!(max_attempts >= 1, "need at least one attempt");
    let mut attempts = 0;
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(e); // give up with the *last* error
                }
                thread::sleep(delay);
            }
        }
    }
}

/// Exponential backoff: 10ms, 20ms, 40ms ... -- the polite version for real
/// network calls, so a struggling server isn't hammered at a fixed rate.
pub fn backoff_delays(base: Duration, attempts: u32) -> Vec<Duration> {
    (0..attempts.saturating_sub(1))
        .map(|i| base * 2u32.pow(i))
        .collect()
}

/// Task 2: simulated config lookup that always fails.
pub fn get_config_value(key: &str) -> Result<String, String> {
    Err(format!("Key '{key}' not found"))
}

pub fn get_config_with_default(key: &str, default: &str) -> String {
    get_config_value(key).unwrap_or_else(|_| default.to_string())
}

pub fn run() {
    match retry(flaky_operation, 5) {
        Ok(result) => println!("Result: {result}"),
        Err(e) => println!("Failed after retries: {e}"),
    }

    let mut calls = 0;
    let outcome = retry_with_delay(
        || {
            calls += 1;
            if calls < 3 {
                Err(format!("attempt {calls} failed"))
            } else {
                Ok(calls)
            }
        },
        5,
        Duration::ZERO,
    );
    println!("deterministic retry: {outcome:?} after {calls} calls");
    println!(
        "backoff schedule for 5 attempts: {:?}",
        backoff_delays(Duration::from_millis(10), 5)
    );

    let timeout = get_config_with_default("timeout", "30");
    println!("Timeout: {timeout}");
}
