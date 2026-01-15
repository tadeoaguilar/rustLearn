# Exercises: Async/Await

**Setup**: Add to Cargo.toml
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

## Exercise 1: Basic Async Functions

**Difficulty**: Easy
**Time**: 20 minutes

```rust
use tokio::time::{sleep, Duration};

async fn say_hello() {
    println!("Hello");
}

async fn say_world() {
    sleep(Duration::from_millis(100)).await;
    println!("World");
}

#[tokio::main]
async fn main() {
    say_hello().await;
    say_world().await;
}
```

**Tasks**:
1. Create async functions that return values
2. Chain multiple async calls
3. Handle errors in async functions

---

## Exercise 2: Concurrent Execution

**Difficulty**: Medium
**Time**: 30 minutes

```rust
use tokio::time::{sleep, Duration};

async fn task_one() -> String {
    sleep(Duration::from_secs(1)).await;
    "Task 1 complete".to_string()
}

async fn task_two() -> String {
    sleep(Duration::from_secs(2)).await;
    "Task 2 complete".to_string()
}

#[tokio::main]
async fn main() {
    // Sequential (slow)
    let start = std::time::Instant::now();
    let r1 = task_one().await;
    let r2 = task_two().await;
    println!("Sequential: {:?}", start.elapsed());

    // Concurrent (fast)
    let start = std::time::Instant::now();
    let (r1, r2) = tokio::join!(task_one(), task_two());
    println!("Concurrent: {:?}", start.elapsed());
}
```

**Tasks**:
1. Run 10 tasks concurrently with `join!`
2. Use `select!` to get first completed task
3. Implement timeout with `tokio::time::timeout`

---

## Exercise 3: Async File I/O

**Difficulty**: Medium
**Time**: 35 minutes

```rust
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Write to file
    let mut file = File::create("test.txt").await?;
    file.write_all(b"Hello, async world!").await?;

    // Read from file
    let mut file = File::open("test.txt").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    println!("Contents: {}", contents);

    Ok(())
}
```

**Tasks**:
1. Copy a file asynchronously
2. Read multiple files concurrently
3. Process large file line-by-line

---

## Exercise 4: Async HTTP Client

**Difficulty**: Medium
**Time**: 40 minutes

**Add dependency**:
```toml
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Post {
    userId: i32,
    id: i32,
    title: String,
    body: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://jsonplaceholder.typicode.com/posts/1";

    let post: Post = reqwest::get(url)
        .await?
        .json()
        .await?;

    println!("{:#?}", post);

    Ok(())
}
```

**Tasks**:
1. Fetch multiple URLs concurrently
2. Implement retry logic
3. Add timeout and error handling

---

## Exercise 5: Channels

**Difficulty**: Medium
**Time**: 35 minutes

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    // Spawn sender task
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
        }
    });

    // Receive messages
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
}
```

**Tasks**:
1. Multiple producers, one consumer
2. Broadcast channel example
3. Oneshot channel for single message

---

## Exercise 6: Spawning Tasks

**Difficulty**: Medium
**Time**: 40 minutes

```rust
use tokio::task;

async fn compute(n: u64) -> u64 {
    // Simulate CPU-intensive work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    n * n
}

#[tokio::main]
async fn main() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = task::spawn(async move {
            compute(i).await
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(result) => println!("Result: {}", result),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }
}
```

**Tasks**:
1. Task that can be cancelled
2. JoinSet for managing multiple tasks
3. Limit concurrent tasks with semaphore

---

## Exercise 7: Async Web Server

**Difficulty**: Hard
**Time**: 60 minutes

**Add dependency**:
```toml
axum = "0.7"
```

```rust
use axum::{
    routing::get,
    Router,
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    content: String,
}

async fn handler() -> Json<Message> {
    Json(Message {
        content: "Hello, async!".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}
```

**Tasks**:
1. Add POST endpoint
2. Add database mock
3. Implement middleware
4. Add error handling

---

## Exercise 8: Streams

**Difficulty**: Hard
**Time**: 45 minutes

```rust
use tokio_stream::{self as stream, StreamExt};

#[tokio::main]
async fn main() {
    let mut stream = stream::iter(vec![1, 2, 3, 4, 5]);

    while let Some(value) = stream.next().await {
        println!("Value: {}", value);
    }

    // Transform stream
    let doubled = stream::iter(vec![1, 2, 3])
        .map(|x| x * 2);

    tokio::pin!(doubled);

    while let Some(value) = doubled.next().await {
        println!("Doubled: {}", value);
    }
}
```

**Tasks**:
1. Create stream from interval
2. Filter and map streams
3. Merge multiple streams

---

## Bonus Challenge: Async Task Queue

**Difficulty**: Very Hard
**Time**: 90+ minutes

Build a task queue system with workers.

```rust
use tokio::sync::mpsc;
use tokio::task;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Job {
    id: u64,
    data: String,
}

async fn worker(id: usize, mut rx: mpsc::Receiver<Job>) {
    while let Some(job) = rx.recv().await {
        println!("Worker {} processing job {}", id, job.id);

        // Simulate work
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("Worker {} completed job {}", id, job.id);
    }
    println!("Worker {} shutting down", id);
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(100);

    // Spawn workers
    let num_workers = 4;
    let mut handles = vec![];

    for i in 0..num_workers {
        let worker_rx = rx.clone();
        let handle = task::spawn(async move {
            worker(i, worker_rx).await;
        });
        handles.push(handle);
    }

    drop(rx);  // Drop original receiver

    // Send jobs
    for i in 0..20 {
        let job = Job {
            id: i,
            data: format!("Job {}", i),
        };
        tx.send(job).await.unwrap();
    }

    drop(tx);  // Signal completion

    // Wait for workers
    for handle in handles {
        handle.await.unwrap();
    }
}
```

**Tasks**:
1. Add priority queue
2. Implement retry logic
3. Add job result collection
4. Create rate limiting

---

## Check Your Understanding

- [ ] Write async functions
- [ ] Use await properly
- [ ] Run tasks concurrently
- [ ] Work with async I/O
- [ ] Use channels for communication
- [ ] Spawn and manage tasks
- [ ] Understand Futures and Streams
- [ ] Handle errors in async code

---

## Common Mistakes

1. **Forgetting .await**: Futures are lazy!
2. **Blocking in async**: Don't use std::thread::sleep
3. **Not handling errors**: Use ? or match
4. **Over-spawning**: Too many tasks can hurt performance
5. **Channel deadlocks**: Drop senders when done

---

## Additional Resources

- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [async-std Book](https://book.async.rs/)
