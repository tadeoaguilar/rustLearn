# Exercises: Concurrency

Rust's promise is "fearless concurrency": data races are compile errors. The
`Send` and `Sync` traits, checked by the compiler, decide what may cross or be
shared between threads. What the compiler *can't* prevent — deadlocks, race
conditions in your logic, contention — is what these exercises practise.

**Setup**: `rayon` and `crossbeam` are already in the exercise crate.

---

## Exercise 1: Threads and Scoped Threads

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Spawn threads, move data in, get results out through `join`
- Borrow local data from threads with `std::thread::scope`

**Tasks**:
1. `spawn_and_collect(n) -> Vec<String>`: spawn `n` named threads (use
   `thread::Builder::new().name(...)`), each returning
   `"hello from worker-{i}"`; join them in order.
2. `parallel_sum(data: &[u64], threads: usize) -> u64`: split `data` into
   chunks and sum each chunk on its own thread, **borrowing** the slice.
   ```rust
   std::thread::scope(|s| {
       let handles: Vec<_> = data.chunks(chunk_len).map(|c| s.spawn(move || c.iter().sum::<u64>())).collect();
       handles.into_iter().map(|h| h.join().unwrap()).sum()
   })
   ```
   Why does this compile with `scope` but not with `thread::spawn`?
3. `panic_is_contained() -> bool`: show that a panicking thread doesn't kill
   the program — `join()` returns `Err`.

---

## Exercise 2: Message Passing

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Build pipelines with `std::sync::mpsc`
- Know when channels end, and use bounded channels for backpressure

**Task 1**: Word count with a fan-out/fan-in pipeline:
```rust
pub fn word_count_pipeline(documents: Vec<String>, workers: usize) -> BTreeMap<String, usize>
```
- a job channel carries documents to `workers` threads
- each worker counts words in its documents and sends a `HashMap` back on a result channel
- the main thread merges the maps

(Hint: `mpsc::Receiver` can't be cloned — share it as `Arc<Mutex<Receiver<_>>>`,
or give each worker its own channel.)

**Task 2**: Backpressure. With `mpsc::sync_channel(2)`, a fast producer blocks
when the buffer is full. Write `bounded_producer_consumer(items, capacity)`
that returns the maximum number of items ever waiting in the channel, and show
it never exceeds the capacity.

---

## Exercise 3: Shared State — Mutex, RwLock, Condvar

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Share mutable state with `Arc<Mutex<T>>` and `Arc<RwLock<T>>`
- Avoid deadlocks with a global lock order
- Block efficiently with `Condvar`

**Task 1**: A bank with one `Mutex` per account:
```rust
pub struct Bank { accounts: Vec<Mutex<i64>> }
impl Bank {
    pub fn transfer(&self, from: usize, to: usize, amount: i64) -> Result<(), BankError>;
    pub fn total(&self) -> i64;
}
```
Run 8 threads doing 1,000 random transfers each. The total must never change,
and the program must not deadlock. Explain the deadlock in the naive version
(lock `from`, then lock `to`) and fix it by always locking the lower index first.

**Task 2**: A `BlockingQueue<T>` with a capacity, built from
`Mutex<VecDeque<T>>` and two `Condvar`s (`not_empty`, `not_full`):
```rust
pub fn push(&self, item: T);   // waits while full
pub fn pop(&self) -> T;        // waits while empty
```
**Always wait in a `while` loop** — condition variables can wake up spuriously.

---

## Exercise 4: Atomics and Memory Ordering

**Difficulty**: Hard
**Time**: 45 minutes

**Learning Objectives**:
- Use `AtomicUsize`/`AtomicBool` instead of a lock for simple shared values
- Choose `Relaxed`, `Acquire`/`Release` and `SeqCst` deliberately

**Tasks**:
1. Count to 1,000,000 from 8 threads three ways: a plain `usize` behind
   `Mutex`, an `AtomicUsize` with `fetch_add(1, Relaxed)`, and per-thread local
   counts summed at the end. Time them.
2. `atomic_max(target: &AtomicU64, value: u64)` with a `compare_exchange` loop
   (then compare with `fetch_max`).
3. A spin lock:
   ```rust
   pub struct SpinLock<T> { locked: AtomicBool, value: UnsafeCell<T> }
   unsafe impl<T: Send> Sync for SpinLock<T> {}
   pub fn lock(&self) -> SpinGuard<'_, T>;   // Acquire on lock, Release on unlock (in Drop)
   ```
   Why is `Relaxed` *wrong* for the lock and unlock?

---

## Exercise 5: A Thread Pool

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Build the Rust Book's chapter 21 thread pool, then improve it
- Shut down gracefully in `Drop`
- Survive panicking jobs

**Task**:
```rust
pub struct ThreadPool { workers: Vec<Worker>, sender: Option<mpsc::Sender<Job>> }
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool;            // panics if size == 0
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F);
    pub fn spawn<F, T>(&self, f: F) -> mpsc::Receiver<T>  // a result you can wait for
        where F: FnOnce() -> T + Send + 'static, T: Send + 'static;
}
impl Drop for ThreadPool { ... }  // drop the sender, then join every worker
```

Requirements:
- `Drop` waits for all queued jobs to finish
- a job that panics doesn't kill its worker (`std::panic::catch_unwind`)
- jobs really run in parallel: 4 jobs of 100 ms on 4 workers take ~100 ms

---

## Exercise 6: Data Parallelism with Rayon

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Turn `iter()` into `par_iter()` and know when it helps
- Process many files in parallel

**Tasks**:
1. `sum_of_squares_par(v: &[u64]) -> u64` and `count_primes_par(limit: u64)`
2. **Parallel file processor**: given a directory, count lines, words and
   bytes of every `.txt` file in parallel; return per-file stats sorted by
   name and the grand total.
   ```rust
   pub fn process_dir(dir: &Path) -> io::Result<(Vec<FileStats>, FileStats)>;
   ```
3. Time sequential vs parallel versions on a big input (in `--release`!).

---

## Exercise 7: A Concurrent Web Crawler

**Difficulty**: Very Hard
**Time**: 90 minutes

**Learning Objectives**:
- Coordinate a dynamic amount of work across threads
- Know when parallel work is *finished*

The web is simulated so the exercise is deterministic and offline:
```rust
pub struct FakeWeb { pages: HashMap<String, Vec<String>>, latency: Duration }
impl FakeWeb {
    pub fn fetch(&self, url: &str) -> Option<Vec<String>>;   // sleeps `latency`, returns links
}
pub fn crawl(web: Arc<FakeWeb>, start: &str, max_depth: usize, workers: usize) -> BTreeSet<String>;
```

Requirements:
- `workers` threads take URLs from a shared queue (`crossbeam::channel` is
  multi-consumer)
- a shared `visited` set means each page is fetched **once**, even if many pages link to it
- links deeper than `max_depth` aren't followed
- the crawl ends when the queue is empty **and** no worker is busy — count
  in-flight work with an atomic

**Test**: 4 workers crawling 40 pages with 10 ms latency should take ~100 ms, not 400 ms.

---

## Exercise 8: A Lock-Free Stack

**Difficulty**: Very Hard
**Time**: 90 minutes

**Learning Objectives**:
- Build a Treiber stack with compare-and-swap
- Understand why lock-free code needs safe memory reclamation

**Task**: implement with `crossbeam::epoch`:
```rust
pub struct LockFreeStack<T> { head: epoch::Atomic<Node<T>> }
struct Node<T> { value: ManuallyDrop<T>, next: epoch::Atomic<Node<T>> }

impl<T> LockFreeStack<T> {
    pub fn push(&self, value: T);
    pub fn pop(&self) -> Option<T>;
    pub fn is_empty(&self) -> bool;
}
```
Push and pop are loops: read `head`, build the new state, `compare_exchange`;
retry if another thread got there first.

**Why epoch?** After `pop` unlinks a node, another thread may still be reading
it. Freeing it immediately is a use-after-free. `guard.defer_destroy(node)`
frees it only once no thread can still hold a reference.

**Then**: compare with `crossbeam::queue::ArrayQueue` (bounded) and `SegQueue`
(unbounded) — production-quality lock-free queues. 8 threads × 10,000 pushes
and pops: every value comes out exactly once.

---

## Bonus Challenge: Dining Philosophers

**Difficulty**: Medium
**Time**: 45 minutes

Five philosophers, five forks (`Mutex<()>`), each needs both neighbours' forks
to eat. Make each philosopher eat 100 times.

1. The naive version — everyone picks up the left fork first — can deadlock.
   (Don't run it without a timeout.)
2. Fix it with **resource ordering**: always pick up the lower-numbered fork first.
3. Return how many times each philosopher ate.

---

## Check Your Understanding

- [ ] Explain `Send` and `Sync` and give a type that is neither
- [ ] Use `thread::scope` to borrow from parallel threads
- [ ] Build a channel pipeline and know when it ends
- [ ] Prevent a deadlock with a lock order
- [ ] Use a `Condvar` correctly (in a loop)
- [ ] Pick a memory ordering and justify it
- [ ] Write a thread pool with graceful shutdown
- [ ] Use rayon for data parallelism
- [ ] Explain why lock-free data structures need epochs or hazard pointers

---

## Additional Resources

- [Rust Book Chapter 16: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rust Book Chapter 21: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html)
- [Rust Atomics and Locks](https://marabos.nl/atomics/) — Mara Bos, free online
- [Rayon](https://docs.rs/rayon/)
- [crossbeam](https://docs.rs/crossbeam/)
