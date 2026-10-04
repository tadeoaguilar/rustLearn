//! Exercise 7: benchmark the two word counters.
//!
//!     cargo bench -p m09-testing
//!
//! See exercises.md for the skeleton of a criterion benchmark.

use criterion::{Criterion, criterion_group, criterion_main};

fn bench(_c: &mut Criterion) {
    // TODO: benchmark m09_testing::words::count_words_split and
    // count_words_borrowed on a large text. Remember std::hint::black_box.
}

criterion_group!(benches, bench);
criterion_main!(benches);
