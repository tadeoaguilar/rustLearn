//! Exercise 8: Streams -- the async version of an iterator.
//!
//! `Iterator::next()` returns `Option<T>`; `Stream::next()` returns a future
//! of `Option<T>`, because the next item may not exist *yet*.

use futures::stream::{self as fstream, Stream};
use tokio::time::{Duration, interval};
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::{self as stream, StreamExt};

/// The exercise's example.
pub async fn iterate_and_double() -> (Vec<i32>, Vec<i32>) {
    let mut s = stream::iter(vec![1, 2, 3, 4, 5]);
    let mut values = Vec::new();
    while let Some(v) = s.next().await {
        values.push(v);
    }
    // `map` returns an adaptor that's not Unpin; `next()` needs it pinned.
    // `collect` would avoid the question, but `pin!` is worth seeing once.
    let doubled = stream::iter(vec![1, 2, 3]).map(|x| x * 2);
    tokio::pin!(doubled);
    let mut d = Vec::new();
    while let Some(v) = doubled.next().await {
        d.push(v);
    }
    (values, d)
}

/// Task 1: a stream from an interval -- one item (an Instant) per tick.
/// Returns when each tick fired, relative to the start. The first tick of an
/// interval fires immediately: [0ms, 20ms, 40ms, ...].
pub async fn ticks(n: usize, every: Duration) -> Vec<Duration> {
    let start = tokio::time::Instant::now();
    IntervalStream::new(interval(every))
        .map(|tick| tick - start)
        .take(n)
        .collect()
        .await
}

/// Task 2: filter and map, exactly like iterators.
pub async fn even_squares(up_to: u32) -> Vec<u32> {
    stream::iter(1..=up_to)
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .collect()
        .await
}

/// Task 3: merge -- items from both streams as they arrive. Here: a fast
/// stream (every 10ms) and a slow one (every 25ms), tagged.
pub async fn merged(n: usize) -> Vec<&'static str> {
    let fast = IntervalStream::new(interval(Duration::from_millis(10))).map(|_| "fast");
    let slow = IntervalStream::new(interval(Duration::from_millis(25))).map(|_| "slow");
    fast.merge(slow).take(n).collect().await
}

/// Writing your own stream without implementing `poll_next` by hand:
/// `unfold` threads a state through an async closure.
pub fn countdown(from: u32) -> impl Stream<Item = u32> {
    fstream::unfold(from, |n| async move {
        if n == 0 {
            None
        } else {
            tokio::time::sleep(Duration::from_millis(5)).await;
            Some((n, n - 1)) // (item to yield, next state)
        }
    })
}

pub async fn run() {
    let (values, doubled) = iterate_and_double().await;
    println!("values {values:?}, doubled {doubled:?}");
    println!(
        "interval ticks at: {:?}",
        ticks(5, Duration::from_millis(20)).await
    );
    println!("even squares: {:?}", even_squares(10).await);
    println!("merged fast/slow: {:?}", merged(8).await);
    let c: Vec<u32> = countdown(5).collect().await;
    println!("countdown via unfold: {c:?}");
}
