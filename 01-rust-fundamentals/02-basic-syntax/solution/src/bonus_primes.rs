//! Bonus Challenge: Prime Number Generator.

/// Trial division, only by odd numbers up to sqrt(n). O(sqrt n) per call.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }
    let mut d = 3;
    // `d * d <= n` instead of `d <= sqrt(n)` avoids floating point entirely.
    while d * d <= n {
        if n % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

/// Sieve of Eratosthenes: every number up to `limit` is crossed out by its
/// prime factors once. O(n log log n) for *all* primes, versus O(n sqrt n) for
/// calling `is_prime` on each number.
pub fn primes_up_to(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }
    let limit = limit as usize;
    let mut is_composite = vec![false; limit + 1];
    let mut primes = Vec::new();

    for n in 2..=limit {
        if is_composite[n] {
            continue;
        }
        primes.push(n as u64);
        // Start at n*n: smaller multiples were crossed out by smaller primes.
        let mut multiple = n * n;
        while multiple <= limit {
            is_composite[multiple] = true;
            multiple += n;
        }
    }
    primes
}

/// The nth prime, 1-based: nth_prime(1) == 2.
///
/// Sieves up to an upper bound for the nth prime (n(ln n + ln ln n) holds for
/// n >= 6), so it never needs to restart.
pub fn nth_prime(n: usize) -> u64 {
    assert!(n >= 1, "primes are numbered from 1");
    let bound = if n < 6 {
        15
    } else {
        let n = n as f64;
        (n * (n.ln() + n.ln().ln())).ceil() as u64
    };
    primes_up_to(bound)[n - 1]
}

pub fn run() {
    println!("primes up to 50: {:?}", primes_up_to(50));
    println!("10th prime: {}", nth_prime(10));
    println!("10,000th prime: {}", nth_prime(10_000));
    println!("is_prime(1_000_000_007) = {}", is_prime(1_000_000_007));
}
