//! Exercise 7: Error Recovery.

use rand::Rng;
use std::thread;
use std::time::Duration;

/// Fails 70% of the time.
pub fn flaky_operation() -> Result<String, &'static str> {
    todo!("Exercise 7")
}

/// Task 1 as in the exercise. Note that `max_retries` is really the maximum
/// number of *attempts*: with 5 the operation runs at most 5 times.
pub fn retry<F, T, E>(operation: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    todo!("Exercise 7")
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
    todo!("Exercise 7")
}

/// Exponential backoff: 10ms, 20ms, 40ms ... -- the polite version for real
/// network calls, so a struggling server isn't hammered at a fixed rate.
pub fn backoff_delays(base: Duration, attempts: u32) -> Vec<Duration> {
    todo!("Exercise 7")
}

/// Task 2: simulated config lookup that always fails.
pub fn get_config_value(key: &str) -> Result<String, String> {
    todo!("Exercise 7")
}

pub fn get_config_with_default(key: &str, default: &str) -> String {
    todo!("Exercise 7")
}

pub fn run() {
    todo!("Exercise 7")
}
