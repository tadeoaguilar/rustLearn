//! Exercise 5: Custom Iterators.
//!
//! Implement one method -- `next` -- and you get ~75 adaptors for free.

/// The exercise's Fibonacci. Changed: it returns None instead of overflowing.
/// The original computes `current + self.next` unconditionally, which panics
/// in a debug build at the 94th number.
#[derive(Debug, Clone)]
pub struct Fibonacci {
    curr: Option<u64>,
    next: Option<u64>,
}

impl Fibonacci {
    pub fn new() -> Self {
        Fibonacci {
            curr: Some(0),
            next: Some(1),
        }
    }
}

impl Default for Fibonacci {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let current = self.curr?;
        self.curr = self.next;
        // None once the sum no longer fits; the iterator then yields the last
        // representable value and stops.
        self.next = self.next.and_then(|n| current.checked_add(n));
        Some(current)
    }
}

/// Task 1: a Range-like iterator with a step, counting up or down.
#[derive(Debug, Clone)]
pub struct StepRange {
    current: i64,
    end: i64,
    step: i64,
}

impl StepRange {
    /// Like `start..end` with a step. A negative step counts down.
    ///
    /// # Panics
    /// If `step` is zero (that range would never end).
    pub fn new(start: i64, end: i64, step: i64) -> Self {
        assert!(step != 0, "step must not be zero");
        StepRange {
            current: start,
            end,
            step,
        }
    }
}

impl Iterator for StepRange {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        let in_range = if self.step > 0 {
            self.current < self.end
        } else {
            self.current > self.end
        };
        if !in_range {
            return None;
        }
        let value = self.current;
        self.current += self.step;
        Some(value)
    }
}

/// Task 2: cycles through a slice forever. It *borrows* the slice, so it needs
/// a lifetime: the iterator can't outlive the data. Yields `&'a T` -- items
/// that stay valid even after the iterator is gone.
#[derive(Debug, Clone)]
pub struct CycleSlice<'a, T> {
    items: &'a [T],
    index: usize,
}

impl<'a, T> CycleSlice<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        CycleSlice { items, index: 0 }
    }
}

impl<'a, T> Iterator for CycleSlice<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.items.is_empty() {
            return None; // otherwise index % 0 would panic
        }
        let item = &self.items[self.index];
        self.index = (self.index + 1) % self.items.len();
        Some(item)
    }
}

/// Task 3: an infinite prime generator. Keeps the primes found so far and
/// tests each candidate only against primes up to its square root.
#[derive(Debug, Default, Clone)]
pub struct Primes {
    found: Vec<u64>,
}

impl Primes {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Iterator for Primes {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let mut candidate = match self.found.last() {
            None => 2,
            Some(2) => 3,
            Some(&p) => p + 2,
        };
        loop {
            let is_prime = self
                .found
                .iter()
                .take_while(|&&p| p * p <= candidate)
                .all(|&p| candidate % p != 0);
            if is_prime {
                self.found.push(candidate);
                return Some(candidate);
            }
            candidate += 2;
        }
    }
}

pub fn run() {
    let fib: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("Fibonacci: {fib:?}");
    println!(
        "Fibonacci never overflows: {} numbers, last {:?}",
        Fibonacci::new().count(),
        Fibonacci::new().last()
    );
    println!(
        "StepRange(0, 10, 3): {:?}",
        StepRange::new(0, 10, 3).collect::<Vec<_>>()
    );
    println!(
        "StepRange(10, 0, -4): {:?}",
        StepRange::new(10, 0, -4).collect::<Vec<_>>()
    );
    let colors = ["red", "green", "blue"];
    println!(
        "cycle: {:?}",
        CycleSlice::new(&colors).take(7).collect::<Vec<_>>()
    );
    println!("primes: {:?}", Primes::new().take(15).collect::<Vec<_>>());
    println!("1000th prime: {:?}", Primes::new().nth(999));
}
