// Reference solution for 36-performance-optimization.
//
//     cargo run --release -p m36-performance-optimization-solution -- <1-4|bonus|all>
//
// Timings only mean something with --release (debug builds don't optimize).

use std::hint::black_box;
use std::time::{Duration, Instant};

use m36_performance_optimization_solution::{
    bonus_swar, ex01_hotpath, ex02_simd, ex03_cache, ex04_branches,
};

/// The best of `runs` timings of `f` (the minimum is the least noisy).
fn time<R>(runs: u32, mut f: impl FnMut() -> R) -> Duration {
    (0..runs)
        .map(|_| {
            let start = Instant::now();
            black_box(f());
            start.elapsed()
        })
        .min()
        .unwrap()
}

fn ex1() {
    println!("--- 1: the hot path (top 5 words of 1,000,000)");
    let text = ex01_hotpath::sample_text(1_000_000);
    let naive = time(3, || ex01_hotpath::top_words_naive(&text, 5));
    let fast = time(3, || ex01_hotpath::top_words_fast(&text, 5));
    assert_eq!(
        ex01_hotpath::top_words_naive(&text, 5),
        ex01_hotpath::top_words_fast(&text, 5)
    );
    println!(
        "naive {naive:?}, fast {fast:?} ({:.1}x)",
        naive.as_secs_f64() / fast.as_secs_f64()
    );
    println!("{:?}", ex01_hotpath::top_words_fast(&text, 5));
}

fn ex2() {
    println!("--- 2: SIMD (this CPU: {})", ex02_simd::simd_level());
    let n = 1 << 20;
    let a: Vec<f32> = (0..n).map(|i| (i % 7) as f32 * 0.5).collect();
    let b: Vec<f32> = (0..n).map(|i| (i % 5) as f32 * 0.25).collect();
    let scalar = time(20, || ex02_simd::dot_scalar(&a, &b));
    let unrolled = time(20, || ex02_simd::dot_unrolled(&a, &b));
    let simd = time(20, || ex02_simd::dot_simd(&a, &b));
    println!("dot of 1M floats: scalar {scalar:?}, unrolled {unrolled:?}, intrinsics {simd:?}");
    let hay: Vec<u8> = (0..16 << 20)
        .map(|i: u32| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
        .collect();
    let scalar = time(5, || ex02_simd::count_byte_scalar(&hay, b'x'));
    let simd = time(5, || ex02_simd::count_byte_simd(&hay, b'x'));
    let swar = time(5, || bonus_swar::count_byte(&hay, b'x'));
    println!("count a byte in 16 MiB: scalar {scalar:?}, SSE2 {simd:?}, SWAR {swar:?}");
}

fn ex3() {
    println!("--- 3: matrix multiply, 512 x 512");
    let (a, b) = (
        ex03_cache::Matrix::sample(512, 1),
        ex03_cache::Matrix::sample(512, 2),
    );
    let naive = time(1, || ex03_cache::multiply_naive(&a, &b));
    let ikj = time(3, || ex03_cache::multiply_ikj(&a, &b));
    let transposed = time(3, || ex03_cache::multiply_transposed(&a, &b));
    let blocked = time(3, || ex03_cache::multiply_blocked(&a, &b, 64));
    println!("i,j,k {naive:?}; i,k,j {ikj:?}; transposed {transposed:?}; 64x64 tiles {blocked:?}");
}

fn ex4() {
    println!("--- 4: branches");
    let values = ex04_branches::random_values(10_000_000, 1000, 7);
    let mut sorted = values.clone();
    sorted.sort_unstable();
    for (name, data) in [("random", &values), ("sorted", &sorted)] {
        let branchy = time(5, || ex04_branches::count_below_branchy(data, 500));
        let branchless = time(5, || ex04_branches::count_below_branchless(data, 500));
        println!("{name} data: branchy {branchy:?}, branchless {branchless:?}");
    }
    let big: Vec<u64> = (0..10_000_000).collect();
    let indexed = time(5, || ex04_branches::sum_indexed(&big));
    let iter = time(5, || ex04_branches::sum_iter(&big));
    println!("sum 10M: indexed {indexed:?}, iterator {iter:?}");
}

fn bonus() {
    println!("--- bonus: SWAR");
    let word = u64::from_le_bytes(*b"ab\0cdefg");
    println!(
        "zero byte in \"ab\\0cdefg\": {} (mask {:#018x})",
        bonus_swar::has_zero_byte(word),
        bonus_swar::zero_byte_mask(word)
    );
    println!(
        "find 'q' in \"the quick brown fox\": {:?}",
        bonus_swar::find_byte(b"the quick brown fox", b'q')
    );
}

fn main() {
    if cfg!(debug_assertions) {
        println!("(debug build: timings are meaningless -- add --release)\n");
    }
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            bonus();
        }
        _ => println!(
            "36-performance-optimization -- reference solution\n\n  cargo run --release -p m36-performance-optimization-solution -- <1-4|bonus|all>"
        ),
    }
}
