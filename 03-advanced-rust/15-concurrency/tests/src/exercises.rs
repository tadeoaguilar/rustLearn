//! Concurrency bugs often *hang* instead of failing. Every test that could
//! deadlock runs under `deadline`: the work happens on a separate thread, and
//! if it doesn't finish in time the test fails with a message instead of
//! freezing the whole test run. (The stuck thread is abandoned; it dies when
//! the test process exits.)

use crate::sut::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn deadline<T: Send + 'static>(limit: Duration, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(limit) {
        Ok(v) => v,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("did not finish within {limit:?} -- deadlock or lost wake-up?")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => panic!("the code under test panicked"),
    }
}

const SECS: fn(u64) -> Duration = Duration::from_secs;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_named_threads_in_order() {
    assert_eq!(
        ex01_threads::spawn_and_collect(3),
        vec![
            "hello from worker-0",
            "hello from worker-1",
            "hello from worker-2"
        ]
    );
}

#[test]
fn ex1_parallel_sum_borrows_and_matches() {
    let data: Vec<u64> = (1..=10_001).collect();
    let expected: u64 = data.iter().sum();
    for threads in [1, 2, 3, 8, 64] {
        assert_eq!(
            ex01_threads::parallel_sum(&data, threads),
            expected,
            "{threads} threads"
        );
    }
    assert_eq!(ex01_threads::parallel_sum(&[], 4), 0);
    assert_eq!(
        ex01_threads::parallel_sum(&[5], 0),
        5,
        "0 threads is treated as 1"
    );
}

#[test]
fn ex1_panics_stay_in_their_thread() {
    assert!(ex01_threads::panic_is_contained());
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_word_count_pipeline() {
    let counts = deadline(SECS(5), || {
        ex02_channels::word_count_pipeline(ex02_channels::sample_documents(), 3)
    });
    assert_eq!(counts.get("the"), Some(&4), "case-insensitive");
    assert_eq!(counts.get("dog"), Some(&3));
    assert_eq!(counts.get("barks"), Some(&1));
    assert_eq!(counts.values().sum::<usize>(), 19);
    // Same answer whatever the number of workers:
    let one = deadline(SECS(5), || {
        ex02_channels::word_count_pipeline(ex02_channels::sample_documents(), 1)
    });
    assert_eq!(one, counts);
    let empty = deadline(SECS(5), || ex02_channels::word_count_pipeline(vec![], 4));
    assert!(empty.is_empty());
}

#[test]
fn ex2_bounded_channels_apply_backpressure() {
    let bounded = deadline(SECS(10), || ex02_channels::producer_lead(100, Some(2)));
    assert!(
        bounded <= 3,
        "sync_channel(2) let the producer get {bounded} ahead"
    );
    let rendezvous = deadline(SECS(10), || ex02_channels::producer_lead(50, Some(0)));
    assert!(
        rendezvous <= 1,
        "sync_channel(0) let the producer get {rendezvous} ahead"
    );
    let unbounded = deadline(SECS(10), || ex02_channels::producer_lead(100, None));
    assert!(
        unbounded > 10,
        "an unbounded channel should let a fast producer race ahead (got {unbounded})"
    );
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_transfers_and_errors() {
    use ex03_shared_state::{Bank, BankError};
    let bank = Bank::new(&[100, 50]);
    assert_eq!(bank.transfer(0, 1, 30), Ok(()));
    assert_eq!(bank.transfer(1, 0, 10), Ok(()));
    assert_eq!(bank.total(), 150);
    assert_eq!(
        bank.transfer(1, 0, 1_000),
        Err(BankError::InsufficientFunds {
            balance: 70,
            requested: 1_000
        })
    );
    assert_eq!(bank.transfer(0, 0, 1), Err(BankError::SameAccount));
    assert_eq!(bank.transfer(0, 9, 1), Err(BankError::NoSuchAccount(9)));
    assert_eq!(bank.total(), 150, "failed transfers change nothing");
}

#[test]
fn ex3_concurrent_transfers_conserve_money_and_never_deadlock() {
    let (before, after) = deadline(SECS(20), || ex03_shared_state::stress_bank(5, 8, 2_000));
    assert_eq!(before, after);
}

#[test]
fn ex3_blocking_queue_hands_over_everything_in_order() {
    use ex03_shared_state::BlockingQueue;
    let received = deadline(SECS(10), || {
        let q = Arc::new(BlockingQueue::new(3));
        let producer = {
            let q = Arc::clone(&q);
            std::thread::spawn(move || (0..1_000).for_each(|i| q.push(i)))
        };
        let got: Vec<i32> = (0..1_000).map(|_| q.pop()).collect();
        producer.join().unwrap();
        assert!(q.is_empty());
        got
    });
    assert_eq!(received, (0..1_000).collect::<Vec<_>>());
}

#[test]
fn ex3_push_blocks_when_full() {
    use ex03_shared_state::BlockingQueue;
    let q = Arc::new(BlockingQueue::new(2));
    q.push(1);
    q.push(2);
    let blocked = {
        let q = Arc::clone(&q);
        std::thread::spawn(move || q.push(3))
    };
    std::thread::sleep(Duration::from_millis(50));
    assert!(!blocked.is_finished(), "push on a full queue must wait");
    assert_eq!(q.pop(), 1);
    deadline(SECS(5), move || blocked.join().unwrap());
    assert_eq!(q.len(), 2);
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_all_three_counters_are_exact() {
    let expected = ex04_atomics::THREADS * ex04_atomics::PER_THREAD;
    assert_eq!(
        deadline(SECS(30), || ex04_atomics::count_with_mutex().0),
        expected
    );
    assert_eq!(
        deadline(SECS(30), || ex04_atomics::count_with_atomic().0),
        expected
    );
    assert_eq!(
        deadline(SECS(30), || ex04_atomics::count_with_local_sums().0),
        expected
    );
}

#[test]
fn ex4_atomic_max() {
    use std::sync::atomic::AtomicU64;
    let max = Arc::new(AtomicU64::new(5));
    assert_eq!(
        ex04_atomics::atomic_max(&max, 3),
        5,
        "smaller value: unchanged, returns current"
    );
    assert_eq!(ex04_atomics::atomic_max(&max, 9), 9);
    let handles: Vec<_> = (0..8u64)
        .map(|t| {
            let max = Arc::clone(&max);
            std::thread::spawn(move || {
                (0..1_000).for_each(|i| {
                    ex04_atomics::atomic_max(&max, t * 1_000 + i);
                })
            })
        })
        .collect();
    handles.into_iter().for_each(|h| h.join().unwrap());
    assert_eq!(max.load(Ordering::SeqCst), 7_999);
}

#[test]
fn ex4_spin_lock_is_mutually_exclusive() {
    // If the guard's Drop doesn't unlock, this hangs -- hence the deadline.
    let total = deadline(SECS(20), || {
        let lock = Arc::new(ex04_atomics::SpinLock::new(0u64));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let lock = Arc::clone(&lock);
                std::thread::spawn(move || {
                    for _ in 0..10_000 {
                        *lock.lock() += 1; // read-modify-write: loses updates without real exclusion
                    }
                })
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
        Arc::try_unwrap(lock).ok().unwrap().into_inner()
    });
    assert_eq!(total, 80_000);
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_jobs_run_in_parallel() {
    let elapsed = deadline(SECS(10), || {
        let pool = ex05_thread_pool::ThreadPool::new(4);
        assert_eq!(pool.size(), 4);
        let start = Instant::now();
        let rxs: Vec<_> = (0..4)
            .map(|i| {
                pool.spawn(move || {
                    std::thread::sleep(Duration::from_millis(100));
                    i
                })
            })
            .collect();
        let results: Vec<i32> = rxs.into_iter().map(|rx| rx.recv().unwrap()).collect();
        assert_eq!(results, vec![0, 1, 2, 3]);
        start.elapsed()
    });
    assert!(
        elapsed < Duration::from_millis(300),
        "4 x 100ms on 4 workers took {elapsed:?}"
    );
}

#[test]
fn ex5_drop_waits_for_queued_jobs() {
    let done = deadline(SECS(10), || {
        let done = Arc::new(AtomicUsize::new(0));
        {
            let pool = ex05_thread_pool::ThreadPool::new(2);
            for _ in 0..20 {
                let done = Arc::clone(&done);
                pool.execute(move || {
                    std::thread::sleep(Duration::from_millis(2));
                    done.fetch_add(1, Ordering::SeqCst);
                });
            }
        }
        done.load(Ordering::SeqCst)
    });
    assert_eq!(done, 20);
}

#[test]
fn ex5_workers_survive_panicking_jobs() {
    let answers = deadline(SECS(10), || {
        let pool = ex05_thread_pool::ThreadPool::new(1); // one worker: it must survive
        pool.execute(|| panic!("job failed"));
        let failed = pool.spawn(|| -> i32 { panic!("this result never arrives") });
        let ok = pool.spawn(|| 42);
        (failed.recv().is_err(), ok.recv().unwrap())
    });
    assert_eq!(answers, (true, 42));
}

#[test]
#[should_panic]
fn ex5_zero_threads_is_rejected() {
    ex05_thread_pool::ThreadPool::new(0);
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_parallel_matches_sequential() {
    use ex06_rayon::*;
    let v: Vec<u64> = (0..100_000).collect();
    assert_eq!(sum_of_squares_par(&v), sum_of_squares_seq(&v));
    assert_eq!(count_primes_par(10_000), 1_229);
    assert_eq!(count_primes_seq(10_000), 1_229);
}

#[test]
fn ex6_process_dir() {
    use ex06_rayon::{FileStats, process_dir};
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("b.txt"), "one two\nthree\n").unwrap();
    std::fs::write(dir.path().join("a.txt"), "hello world").unwrap();
    std::fs::write(dir.path().join("ignored.md"), "not counted").unwrap();
    let (files, total) = process_dir(dir.path()).unwrap();
    assert_eq!(
        files,
        vec![
            FileStats {
                name: "a.txt".into(),
                lines: 1,
                words: 2,
                bytes: 11
            },
            FileStats {
                name: "b.txt".into(),
                lines: 2,
                words: 3,
                bytes: 14
            },
        ]
    );
    assert_eq!((total.lines, total.words, total.bytes), (3, 5, 25));
    assert!(process_dir(&dir.path().join("missing")).is_err());
}

// ---- Exercise 7 --------------------------------------------------------------

#[test]
fn ex7_crawl_finds_everything_once() {
    use ex07_crawler::{FakeWeb, crawl};
    let web = Arc::new(FakeWeb::binary_tree(40, Duration::from_millis(1)));
    let pages = deadline(SECS(20), {
        let web = Arc::clone(&web);
        move || crawl(web, "/0", 10, 4)
    });
    assert_eq!(pages.len(), 40, "every existing page reached");
    assert!(!pages.contains("/missing"), "dead links aren't pages");
    assert_eq!(
        web.fetch_count(),
        41,
        "40 pages + /missing once -- each URL fetched exactly once"
    );
}

#[test]
fn ex7_depth_limit() {
    use ex07_crawler::{FakeWeb, crawl};
    let web = Arc::new(FakeWeb::binary_tree(40, Duration::ZERO));
    let pages = deadline(SECS(20), move || crawl(web, "/0", 2, 4));
    let expected: std::collections::BTreeSet<String> = (0..7).map(|i| format!("/{i}")).collect();
    assert_eq!(
        pages, expected,
        "depth 0: /0, depth 1: /1 /2, depth 2: /3../6"
    );
}

#[test]
fn ex7_workers_overlap_the_latency() {
    use ex07_crawler::{FakeWeb, crawl};
    let web = Arc::new(FakeWeb::binary_tree(40, Duration::from_millis(10)));
    let elapsed = deadline(SECS(20), move || {
        let start = Instant::now();
        crawl(web, "/0", 10, 8);
        start.elapsed()
    });
    // 41 fetches x 10ms = 410ms sequentially.
    assert!(
        elapsed < Duration::from_millis(300),
        "8 workers took {elapsed:?}"
    );
}

// ---- Exercise 8 --------------------------------------------------------------

#[test]
fn ex8_stack_is_lifo() {
    let s = ex08_lock_free::LockFreeStack::new();
    assert!(s.is_empty());
    s.push("a");
    s.push("b");
    assert!(!s.is_empty());
    assert_eq!(s.pop(), Some("b"));
    assert_eq!(s.pop(), Some("a"));
    assert_eq!(s.pop(), None);
}

#[test]
fn ex8_every_value_exactly_once_under_contention() {
    let all = deadline(SECS(30), || ex08_lock_free::stress_stack(8, 10_000));
    assert_eq!(all, (0..80_000).collect::<Vec<_>>());
    let all = deadline(SECS(30), || {
        ex08_lock_free::mpmc_with_array_queue(4, 10_000)
    });
    assert_eq!(all, (0..40_000).collect::<Vec<_>>());
}

#[test]
fn ex8_dropping_a_non_empty_stack_drops_its_values() {
    let drops = Arc::new(AtomicUsize::new(0));
    struct D(Arc<AtomicUsize>);
    impl Drop for D {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    {
        let s = ex08_lock_free::LockFreeStack::new();
        for _ in 0..5 {
            s.push(D(Arc::clone(&drops)));
        }
        drop(s.pop()); // 1
    } // 4 more
    assert_eq!(drops.load(Ordering::SeqCst), 5);
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_everyone_eats_and_nobody_deadlocks() {
    let meals = deadline(SECS(30), || bonus_philosophers::dine(5, 200));
    assert_eq!(meals, vec![200; 5]);
}
