// Reference solution for 35-memory-management.
//
//     cargo run -p m35-memory-management-solution -- <1-5|bonus|all>
//     cargo run --release -p m35-memory-management-solution -- 5      (timings mean something in release)

use std::alloc::Layout;
use std::time::Instant;

use m35_memory_management_solution::bonus_arena_tree::Tree;
use m35_memory_management_solution::ex01_bump::Bump;
use m35_memory_management_solution::ex02_pool::{ObjectPool, Slab};
use m35_memory_management_solution::ex03_counting::{self, Counting};
use m35_memory_management_solution::ex04_layout::{self, Packed, Padded};
use m35_memory_management_solution::ex05_soa;

// Every allocation in this program is counted.
#[global_allocator]
static ALLOC: Counting = Counting::system();

fn ex1() {
    println!("--- 1: a bump allocator");
    let mut arena = Bump::new(1024);
    let a = arena.alloc_value(1u8).unwrap();
    *a += 1;
    let b = arena.alloc_value(0xDEAD_BEEF_u64).unwrap();
    println!(
        "a u8 then a u64: {} bytes used (7 of padding to align the u64): {a}, {b:#x}",
        arena.used()
    );
    let s = arena.alloc_str("allocated in the arena").unwrap();
    println!("a string: {s:?}; used {}", arena.used());
    let huge = arena.alloc(Layout::from_size_align(2000, 8).unwrap());
    println!("2000 more bytes: {huge:?} (doesn't fit)");
    arena.reset();
    println!("after reset: {} used", arena.used());
    let (_, m) = ex03_counting::measure(|| {
        for i in 0..1000u64 {
            arena.alloc_value(i);
        }
    });
    println!(
        "1000 arena allocations made {} heap allocations",
        m.allocations
    );
}

fn ex2() {
    println!("--- 2: pools");
    let mut slab = Slab::new();
    let a = slab.insert("first");
    let b = slab.insert("second");
    slab.remove(a);
    let c = slab.insert("third");
    println!("handles: a {a:?}, b {b:?}, c {c:?} (c reused a's slot, next generation)");
    println!(
        "slab.get(a) after reuse: {:?}; slab.get(c): {:?}",
        slab.get(a),
        slab.get(c)
    );
    let pool: ObjectPool<Vec<u8>> = ObjectPool::new(|| Vec::with_capacity(4096), Vec::clear);
    let (_, m) = ex03_counting::measure(|| {
        for i in 0..1000 {
            let mut buf = pool.take();
            buf.extend_from_slice(format!("request {i}").as_bytes());
        }
    });
    println!(
        "1000 requests with pooled 4 KiB buffers: {} buffer(s) created, {} allocations in total (the format! strings)",
        pool.created(),
        m.allocations
    );
}

fn ex3() {
    println!("--- 3: counting allocations");
    let words: Vec<String> = (0..200).map(|i| format!("word{i}")).collect();
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    let (naive, m1) = ex03_counting::measure(|| ex03_counting::join_naive(&words));
    let (reserved, m2) = ex03_counting::measure(|| ex03_counting::join_reserved(&words));
    assert_eq!(naive, reserved);
    println!(
        "join_naive:    {:>4} allocations, {:>6} bytes requested",
        m1.allocations, m1.bytes
    );
    println!(
        "join_reserved: {:>4} allocations, {:>6} bytes requested",
        m2.allocations, m2.bytes
    );
    let (_, m3) = ex03_counting::measure(|| {
        ex03_counting::count_long_words("the quick brown fox jumps over the lazy dog", 3)
    });
    println!("count_long_words: {} allocations", m3.allocations);
    let s = ALLOC.snapshot();
    println!(
        "whole program so far: {} allocations, {} frees, {} bytes live, peak {} bytes",
        s.allocations, s.deallocations, s.live_bytes, s.peak_bytes
    );
}

fn ex4() {
    println!("--- 4: layout");
    println!(
        "Padded {{ bool, u64, u8 }}: {} bytes; Packed {{ u64, bool, u8 }}: {} bytes",
        size_of::<Padded>(),
        size_of::<Packed>()
    );
    let fields = [(1, 1), (8, 8), (1, 1)];
    println!(
        "layout_c(bool, u64, u8) = {:?}, padding {}",
        ex04_layout::layout_c(&fields),
        ex04_layout::padding(&fields)
    );
    println!("best order: {:?}", ex04_layout::best_order(&fields));
    println!(
        "Option<&u8> free? {}  Option<Box<u8>>? {}  Option<u32>? {}",
        ex04_layout::option_is_free::<&u8>(),
        ex04_layout::option_is_free::<Box<u8>>(),
        ex04_layout::option_is_free::<u32>()
    );
}

fn ex5() {
    println!("--- 5: AoS vs SoA (run with --release for meaningful times)");
    let n = 1_000_000;
    let mut aos = ex05_soa::generate(n);
    let mut soa = aos.to_soa();
    let t = Instant::now();
    for _ in 0..10 {
        aos.step(0.01);
    }
    let aos_time = t.elapsed();
    let t = Instant::now();
    for _ in 0..10 {
        soa.step(0.01);
    }
    println!(
        "10 steps of {n} particles: AoS {aos_time:?}, SoA {:?}",
        t.elapsed()
    );
    println!("same result: {}", aos == soa.to_aos());
    let t = Instant::now();
    let mut a = 0.0;
    for _ in 0..10 {
        a += aos.mean_x();
    }
    let aos_time = t.elapsed();
    let t = Instant::now();
    let mut b = 0.0;
    for _ in 0..10 {
        b += soa.mean_x();
    }
    println!(
        "10 x mean x (one field): AoS {aos_time:?}, SoA {:?} (equal: {})",
        t.elapsed(),
        a == b
    );
    let (rows, cols) = (4096, 4096);
    let data: Vec<u32> = (0..rows * cols).map(|i| i as u32 % 1000).collect();
    let t = Instant::now();
    let a = ex05_soa::sum_row_major(&data, rows, cols);
    let row_time = t.elapsed();
    let t = Instant::now();
    let b = ex05_soa::sum_col_major(&data, rows, cols);
    println!(
        "4096x4096 sum: row order {row_time:?}, column order {:?} (equal: {})",
        t.elapsed(),
        a == b
    );
}

fn bonus() {
    println!("--- bonus: a tree in an arena");
    let mut tree = Tree::new(1u64);
    let root = tree.root();
    let a = tree.add_child(root, 2);
    let b = tree.add_child(root, 3);
    let a1 = tree.add_child(a, 4);
    tree.add_child(b, 5);
    println!(
        "path from {a1:?} to the root: {:?}, depth {}",
        tree.path_to_root(a1),
        tree.depth(a1)
    );
    println!(
        "pre-order: {:?}; sum under the root {}, under {b:?} {}",
        tree.descendants(root),
        tree.subtree_sum(root),
        tree.subtree_sum(b)
    );
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("5") => ex5(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            ex5();
            bonus();
        }
        _ => println!(
            "35-memory-management -- reference solution\n\n  cargo run -p m35-memory-management-solution -- <1-5|bonus|all>"
        ),
    }
}
