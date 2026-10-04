//! Exercise 7: `cargo bench -p m09-testing-solution`
//!
//! Criterion runs each function many times, reports a confidence interval,
//! and on later runs tells you whether performance changed. HTML reports go
//! to target/criterion/report/index.html.

use criterion::{Criterion, criterion_group, criterion_main};
use m09_testing_solution::words::{count_words_borrowed, count_words_split};
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let text = "the quick brown fox jumps over the lazy dog ".repeat(10_000);
    let mut group = c.benchmark_group("count_words");
    // black_box stops the optimiser from computing the result once at compile
    // time, or deleting the call because the result is unused.
    group.bench_function("split (String keys)", |b| {
        b.iter(|| count_words_split(black_box(&text)))
    });
    group.bench_function("borrowed (&str keys)", |b| {
        b.iter(|| count_words_borrowed(black_box(&text)))
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
