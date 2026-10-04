//! Bonus: Dining Philosophers.
//!
//! Naive: philosopher i picks up fork i, then fork i+1. If all five pick up
//! their left fork at the same moment, each waits forever for the right one --
//! a cycle of waiting, i.e. deadlock.
//!
//! Fix: resource ordering. Everyone picks up the *lower-numbered* fork first.
//! Philosopher 4 (forks 4 and 0) now reaches for fork 0 first, which breaks
//! the cycle: at least one philosopher can always get both forks.

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub fn dine(philosophers: usize, meals: usize) -> Vec<usize> {
    let forks: Vec<Mutex<()>> = (0..philosophers).map(|_| Mutex::new(())).collect();
    thread::scope(|s| {
        let handles: Vec<_> = (0..philosophers)
            .map(|i| {
                let forks = &forks;
                s.spawn(move || {
                    let (left, right) = (i, (i + 1) % philosophers);
                    let (first, second) = (left.min(right), left.max(right));
                    let mut eaten = 0;
                    for _ in 0..meals {
                        let _a = forks[first].lock().unwrap();
                        let _b = forks[second].lock().unwrap();
                        eaten += 1; // eat
                        drop((_a, _b)); // put both forks down
                        thread::sleep(Duration::from_micros(50)); // think
                    }
                    eaten
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

pub fn run() {
    println!("5 philosophers x 100 meals: {:?}", dine(5, 100));
}
