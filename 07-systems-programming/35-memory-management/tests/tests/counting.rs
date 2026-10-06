//! Exercise 3 with the counting allocator installed as THE global allocator.
//!
//! A separate test binary (`tests/` directory), because a global allocator
//! applies to the whole program. `measure` counts per thread, so the tests
//! can still run in parallel.

use m35_memory_management_tests::sut::ex03_counting::{self as counting, Counting};

#[global_allocator]
static ALLOC: Counting = Counting::system();

fn words(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("word{i}")).collect()
}

#[test]
fn ex3_measure_counts_this_threads_allocations() {
    let (v, m) = counting::measure(|| Vec::<u8>::with_capacity(1000));
    assert_eq!((m.allocations, m.bytes), (1, 1000));
    drop(v);
    let (_, m) = counting::measure(|| ());
    assert_eq!(m.allocations, 0);
    let (_, m) = counting::measure(|| Box::new([0u64; 4]));
    assert_eq!((m.allocations, m.bytes), (1, 32));
}

#[test]
fn ex3_reserving_saves_allocations() {
    let owned = words(300);
    let words: Vec<&str> = owned.iter().map(String::as_str).collect();
    let (naive, naive_m) = counting::measure(|| counting::join_naive(&words));
    let (reserved, reserved_m) = counting::measure(|| counting::join_reserved(&words));
    assert_eq!(naive, reserved);
    assert_eq!(reserved_m.allocations, 1, "one exact allocation");
    assert!(naive_m.allocations > 3, "growing reallocates: {naive_m:?}");
    assert!(naive_m.bytes > reserved_m.bytes);
}

#[test]
fn ex3_zero_allocation_functions() {
    let text = "a function can do real work without touching the heap at all";
    let (n, m) = counting::measure(|| counting::count_long_words(text, 4));
    assert_eq!(n, 3); // function, without, touching
    assert_eq!(m.allocations, 0);
}

#[test]
fn ex3_global_snapshot_tracks_live_bytes() {
    let before = ALLOC.snapshot();
    assert!(before.allocations > 0, "the test harness itself allocates");
    assert!(before.peak_bytes >= before.live_bytes);
    let big = vec![0u8; 1 << 20];
    let during = ALLOC.snapshot();
    assert!(during.peak_bytes >= 1 << 20);
    assert!(during.allocations > before.allocations);
    drop(big);
    assert!(ALLOC.snapshot().deallocations > before.deallocations);
}
