//! Bonus Challenge: Prime Number Generator.

/// Trial division, only by odd numbers up to sqrt(n). O(sqrt n) per call.
pub fn is_prime(n: u64) -> bool {
    todo!("Bonus")
}

/// Sieve of Eratosthenes: every number up to `limit` is crossed out by its
/// prime factors once. O(n log log n) for *all* primes, versus O(n sqrt n) for
/// calling `is_prime` on each number.
pub fn primes_up_to(limit: u64) -> Vec<u64> {
    todo!("Bonus")
}

/// The nth prime, 1-based: nth_prime(1) == 2.
///
/// Sieves up to an upper bound for the nth prime (n(ln n + ln ln n) holds for
/// n >= 6), so it never needs to restart.
pub fn nth_prime(n: usize) -> u64 {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
