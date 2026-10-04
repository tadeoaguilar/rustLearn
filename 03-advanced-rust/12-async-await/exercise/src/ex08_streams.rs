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
    todo!("Exercise 8")
}

/// Task 1: a stream from an interval -- one item (an Instant) per tick.
/// Returns when each tick fired, relative to the start. The first tick of an
/// interval fires immediately: [0ms, 20ms, 40ms, ...].
pub async fn ticks(n: usize, every: Duration) -> Vec<Duration> {
    todo!("Exercise 8")
}

/// Task 2: filter and map, exactly like iterators.
pub async fn even_squares(up_to: u32) -> Vec<u32> {
    todo!("Exercise 8")
}

/// Task 3: merge -- items from both streams as they arrive. Here: a fast
/// stream (every 10ms) and a slow one (every 25ms), tagged.
pub async fn merged(n: usize) -> Vec<&'static str> {
    todo!("Exercise 8")
}

/// Writing your own stream without implementing `poll_next` by hand:
/// `unfold` threads a state through an async closure.
pub fn countdown(from: u32) -> impl Stream<Item = u32> {
    // A bare `todo!()` doesn't compile behind `impl Stream`; replace both lines.
    todo!("Exercise 8");
    fstream::empty()
}

pub async fn run() {
    todo!("Exercise 8")
}
