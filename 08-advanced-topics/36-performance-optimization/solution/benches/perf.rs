//! Criterion benchmarks (optional): `cargo bench -p m36-performance-optimization-solution`.
//! Criterion runs each function many times, reports confidence intervals,
//! and compares against the previous run -- the right tool for "is it
//! faster?", where a single timing isn't.

use criterion::{Criterion, criterion_group, criterion_main};
use m36_performance_optimization_solution::{ex01_hotpath, ex02_simd, ex03_cache};
use std::hint::black_box;

fn hotpath(c: &mut Criterion) {
    let text = ex01_hotpath::sample_text(100_000);
    let mut g = c.benchmark_group("top_words");
    g.bench_function("naive", |b| {
        b.iter(|| ex01_hotpath::top_words_naive(black_box(&text), 5))
    });
    g.bench_function("fast", |b| {
        b.iter(|| ex01_hotpath::top_words_fast(black_box(&text), 5))
    });
    g.finish();
}

fn simd(c: &mut Criterion) {
    let a: Vec<f32> = (0..65_536).map(|i| i as f32).collect();
    let mut g = c.benchmark_group("dot");
    g.bench_function("scalar", |b| {
        b.iter(|| ex02_simd::dot_scalar(black_box(&a), black_box(&a)))
    });
    g.bench_function("unrolled", |b| {
        b.iter(|| ex02_simd::dot_unrolled(black_box(&a), black_box(&a)))
    });
    g.bench_function("simd", |b| {
        b.iter(|| ex02_simd::dot_simd(black_box(&a), black_box(&a)))
    });
    g.finish();
}

fn matmul(c: &mut Criterion) {
    let (a, m) = (
        ex03_cache::Matrix::sample(128, 1),
        ex03_cache::Matrix::sample(128, 2),
    );
    let mut g = c.benchmark_group("matmul_128");
    g.bench_function("naive", |b| {
        b.iter(|| ex03_cache::multiply_naive(black_box(&a), black_box(&m)))
    });
    g.bench_function("ikj", |b| {
        b.iter(|| ex03_cache::multiply_ikj(black_box(&a), black_box(&m)))
    });
    g.bench_function("blocked", |b| {
        b.iter(|| ex03_cache::multiply_blocked(black_box(&a), black_box(&m), 32))
    });
    g.finish();
}

criterion_group!(benches, hotpath, simd, matmul);
criterion_main!(benches);
