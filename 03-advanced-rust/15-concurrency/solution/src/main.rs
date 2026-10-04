// Reference solution for 15-concurrency.
//
//     cargo run -p m15-concurrency-solution -- 3                # one exercise
//     cargo run -p m15-concurrency-solution --release -- all    # everything; --release for real timings

use m15_concurrency_solution::*;

fn main() {
    let parts: [(&str, &str, fn()); 9] = [
        ("1", "Threads and scoped threads", ex01_threads::run),
        ("2", "Message passing", ex02_channels::run),
        ("3", "Mutex, lock ordering, Condvar", ex03_shared_state::run),
        ("4", "Atomics and memory ordering", ex04_atomics::run),
        ("5", "Thread pool", ex05_thread_pool::run),
        ("6", "Rayon", ex06_rayon::run),
        ("7", "Concurrent crawler", ex07_crawler::run),
        ("8", "Lock-free stack", ex08_lock_free::run),
        ("bonus", "Dining philosophers", bonus_philosophers::run),
    ];

    let choice = std::env::args().nth(1).unwrap_or_default();
    match parts.iter().find(|(key, _, _)| *key == choice) {
        Some((key, title, run)) => {
            println!("=== EXERCISE {key}: {title} ===\n");
            run();
        }
        None if choice == "all" => {
            for (key, title, run) in parts {
                println!("\n=== EXERCISE {key}: {title} ===\n");
                run();
            }
        }
        None => {
            println!("15-concurrency -- reference solution\n");
            for (key, title, _) in parts {
                println!("  cargo run -p m15-concurrency-solution -- {key:<6} {title}");
            }
            println!("  cargo run -p m15-concurrency-solution -- all    Everything");
        }
    }
}
