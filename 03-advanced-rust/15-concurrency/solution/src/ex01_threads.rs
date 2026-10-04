//! Exercise 1: Threads and Scoped Threads.

use std::thread;

/// Named threads; results come back through `join`, in spawn order.
pub fn spawn_and_collect(n: usize) -> Vec<String> {
    let handles: Vec<_> = (0..n)
        .map(|i| {
            thread::Builder::new()
                .name(format!("worker-{i}"))
                .spawn(|| {
                    let name = thread::current().name().unwrap_or("unnamed").to_string();
                    format!("hello from {name}")
                })
                .expect("the OS let us create a thread")
        })
        .collect();
    handles
        .into_iter()
        .map(|h| h.join().expect("worker panicked"))
        .collect()
}

/// `thread::spawn` needs `'static` closures: the thread might outlive this
/// function, so it can't borrow `data`. `thread::scope` guarantees every
/// thread spawned inside it is joined before `scope` returns -- so borrowing
/// is safe, and the compiler allows it.
pub fn parallel_sum(data: &[u64], threads: usize) -> u64 {
    if data.is_empty() {
        return 0;
    }
    let chunk_len = data.len().div_ceil(threads.max(1));
    thread::scope(|s| {
        let handles: Vec<_> = data
            .chunks(chunk_len)
            .map(|chunk| s.spawn(move || chunk.iter().sum::<u64>()))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("summing thread panicked"))
            .sum()
    })
}

/// A panic ends only its own thread; `join` reports it as `Err`.
pub fn panic_is_contained() -> bool {
    let handle = thread::spawn(|| {
        panic!("this thread fails on purpose");
    });
    handle.join().is_err()
}

pub fn run() {
    for line in spawn_and_collect(3) {
        println!("{line}");
    }
    let data: Vec<u64> = (1..=1_000_000).collect();
    println!("parallel_sum on 8 threads = {}", parallel_sum(&data, 8));
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    println!(
        "panic contained to its own thread: {}",
        panic_is_contained()
    );
    std::panic::set_hook(previous);
}
