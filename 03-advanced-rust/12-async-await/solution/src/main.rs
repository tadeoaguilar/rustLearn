// Reference solution for 12-async-await.
//
//     cargo run -p m12-async-await-solution -- 2        # one exercise
//     cargo run -p m12-async-await-solution -- all      # everything (about 20 seconds of timers)
//     cargo run -p m12-async-await-solution -- serve    # Exercise 7's server on :3000, until Ctrl-C
//
// #[tokio::main] builds a multi-threaded runtime and runs this async main on it.

use m12_async_await_solution::*;

const PARTS: [(&str, &str); 9] = [
    ("1", "Basic async functions"),
    ("2", "Concurrent execution"),
    ("3", "Async file I/O"),
    ("4", "HTTP client (against a local mock API)"),
    ("5", "Channels"),
    ("6", "Spawning tasks"),
    ("7", "Web server"),
    ("8", "Streams"),
    ("bonus", "Task queue"),
];

async fn run_part(key: &str) {
    match key {
        "1" => ex01_basics::run().await,
        "2" => ex02_concurrency::run().await,
        "3" => ex03_file_io::run().await,
        "4" => ex04_http_client::run().await,
        "5" => ex05_channels::run().await,
        "6" => ex06_tasks::run().await,
        "7" => ex07_web_server::run().await,
        "8" => ex08_streams::run().await,
        "bonus" => bonus_task_queue::run().await,
        _ => unreachable!(),
    }
}

#[tokio::main]
async fn main() {
    let choice = std::env::args().nth(1).unwrap_or_default();
    if choice == "serve" {
        ex07_web_server::serve_forever().await;
        return;
    }
    if let Some((key, title)) = PARTS.iter().find(|(k, _)| *k == choice) {
        println!("=== EXERCISE {key}: {title} ===\n");
        run_part(key).await;
    } else if choice == "all" {
        for (key, title) in PARTS {
            println!("\n=== EXERCISE {key}: {title} ===\n");
            run_part(key).await;
        }
    } else {
        println!("12-async-await -- reference solution\n");
        for (key, title) in PARTS {
            println!("  cargo run -p m12-async-await-solution -- {key:<6} {title}");
        }
        println!("  cargo run -p m12-async-await-solution -- all    Everything");
        println!(
            "  cargo run -p m12-async-await-solution -- serve  Exercise 7's server on 127.0.0.1:3000"
        );
    }
}
