//! Exercise 1: Threads and Scoped Threads.

use std::thread;

/// Named threads; results come back through `join`, in spawn order.
pub fn spawn_and_collect(n: usize) -> Vec<String> {
    todo!("Exercise 1")
}

/// `thread::spawn` needs `'static` closures: the thread might outlive this
/// function, so it can't borrow `data`. `thread::scope` guarantees every
/// thread spawned inside it is joined before `scope` returns -- so borrowing
/// is safe, and the compiler allows it.
pub fn parallel_sum(data: &[u64], threads: usize) -> u64 {
    todo!("Exercise 1")
}

/// A panic ends only its own thread; `join` reports it as `Err`.
pub fn panic_is_contained() -> bool {
    todo!("Exercise 1")
}

pub fn run() {
    todo!("Exercise 1")
}
