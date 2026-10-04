//! FFI tests can't run under Miri (it can't call C). The pure-Rust ones
//! (ex1_, ex5_, ex6_arena) can:
//!     cargo +nightly miri test -p m14-unsafe-ffi-tests -- ex1_ ex5_ ex6_arena

use crate::sut::*;
use std::cell::Cell;
use std::rc::Rc;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_swap_and_sum() {
    let (mut a, mut b) = (1, 2);
    ex01_raw_pointers::swap_via_pointers(&mut a, &mut b);
    assert_eq!((a, b), (2, 1));
    assert_eq!(
        ex01_raw_pointers::sum_with_pointer_arithmetic(&[1, 2, 3, 4]),
        10
    );
    assert_eq!(ex01_raw_pointers::sum_with_pointer_arithmetic(&[]), 0);
}

#[test]
fn ex1_split_at_mut() {
    let mut v = [1, 2, 3, 4, 5];
    let (left, right) = ex01_raw_pointers::my_split_at_mut(&mut v, 2);
    assert_eq!((left.len(), right.len()), (2, 3));
    left[1] = 20;
    right[0] = 30;
    assert_eq!(v, [1, 20, 30, 4, 5]);
    let (l, r) = ex01_raw_pointers::my_split_at_mut(&mut v, 5);
    assert_eq!((l.len(), r.len()), (5, 0));
}

#[test]
#[should_panic]
fn ex1_split_at_mut_checks_bounds() {
    ex01_raw_pointers::my_split_at_mut(&mut [1, 2], 3);
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_libc() {
    use ex02_libc::*;
    assert_eq!(c_abs(-42), 42);
    assert_eq!(c_abs(7), 7);
    assert_eq!(c_strlen("hello"), Ok(5));
    assert_eq!(c_strlen("héllo"), Ok(6), "bytes, not chars");
    assert_eq!(c_strlen(""), Ok(0));
    assert!(
        c_strlen("a\0b").is_err(),
        "interior NUL can't be a C string"
    );
}

#[test]
fn ex2_qsort() {
    let mut v = [5, -1, 3, 3, 0, i32::MIN, i32::MAX];
    ex02_libc::c_sort(&mut v);
    assert_eq!(v, [i32::MIN, -1, 0, 3, 3, 5, i32::MAX]);
    let mut empty: [i32; 0] = [];
    ex02_libc::c_sort(&mut empty);
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_struct_by_value_and_slices() {
    use ex03_c_library::*;
    let p = |x, y| Point { x, y };
    assert_eq!(distance(p(0.0, 0.0), p(3.0, 4.0)), 5.0);
    assert_eq!(polygon_area(&[p(0.0, 0.0), p(4.0, 0.0), p(0.0, 3.0)]), 6.0);
    assert_eq!(
        polygon_area(&[p(0.0, 0.0), p(1.0, 1.0)]),
        0.0,
        "fewer than 3 points"
    );
    assert_eq!(polygon_area(&[]), 0.0);
}

#[test]
fn ex3_counter_wrapper() {
    use ex03_c_library::Counter;
    let mut c = Counter::new("visits").unwrap();
    assert_eq!(c.get(), 0);
    c.add(3);
    c.add(-1);
    assert_eq!(c.get(), 2);
    assert_eq!(c.name(), "visits");
    let long = "x".repeat(1000);
    assert_eq!(
        Counter::new(&long).unwrap().name(),
        long,
        "the buffer is sized from C's answer"
    );
    assert_eq!(Counter::new("héllo").unwrap().name(), "héllo");
    assert!(Counter::new("bad\0name").is_err());
}

#[test]
fn ex3_many_counters_are_freed() {
    // Not provable without a leak checker, but it mustn't crash or double-free.
    for i in 0..10_000 {
        let mut c = ex03_c_library::Counter::new("tmp").unwrap();
        c.add(i);
    }
}

#[test]
fn ex3_closures_through_c_callbacks() {
    use ex03_c_library::*;
    let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
    let mut seen = Vec::new();
    for_each_point(&points, |p| seen.push(p));
    assert_eq!(seen, points);
    let mut total = 0.0;
    for_each_point(&points, |p| total += p.x + p.y);
    assert_eq!(total, 10.0);
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_checksum_and_c_calling_rust() {
    use ex04_rust_from_c::*;
    assert_eq!(checksum(b""), 0x811c_9dc5, "FNV-1a offset basis");
    assert_eq!(checksum(b"a"), 0xe40c_292c, "FNV-1a test vector");
    // SAFETY: valid pointer/length; null is explicitly allowed.
    unsafe {
        assert_eq!(rust_checksum(b"abc".as_ptr(), 3), checksum(b"abc"));
        assert_eq!(rust_checksum(std::ptr::null(), 10), 0);
    }
    let data = b"hello ffi";
    assert_eq!(
        checksum_twice_via_c(data),
        checksum(data).wrapping_mul(2),
        "C called rust_checksum twice"
    );
}

#[test]
fn ex4_strings_allocated_in_rust() {
    use ex04_rust_from_c::*;
    use std::ffi::{CStr, CString};
    let name = CString::new("Ferris").unwrap();
    // SAFETY: a valid C string.
    let raw = unsafe { rust_greeting(name.as_ptr()) };
    assert!(!raw.is_null(), "rust_greeting returned NULL");
    // SAFETY: non-null pointer from rust_greeting.
    let text = unsafe { CStr::from_ptr(raw) }.to_str().unwrap().to_owned();
    // SAFETY: from rust_greeting, freed once.
    unsafe { rust_free_string(raw) };
    assert_eq!(text, "Hello, Ferris!");
    // SAFETY: null is allowed.
    let raw = unsafe { rust_greeting(std::ptr::null()) };
    assert!(!raw.is_null());
    // SAFETY: as above.
    let text = unsafe { CStr::from_ptr(raw) }.to_str().unwrap().to_owned();
    unsafe { rust_free_string(raw) };
    assert_eq!(text, "Hello, stranger!");
    unsafe { rust_free_string(std::ptr::null_mut()) }; // must be a no-op
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_push_pop_full() {
    let mut v: ex05_stack_vec::StackVec<String, 2> = ex05_stack_vec::StackVec::new();
    assert!(v.is_empty());
    assert_eq!(v.capacity(), 2);
    assert_eq!(v.push("a".into()), Ok(()));
    assert_eq!(v.push("b".into()), Ok(()));
    assert_eq!(
        v.push("c".into()),
        Err("c".to_string()),
        "full: the value comes back"
    );
    assert_eq!(v.as_slice(), ["a", "b"]);
    assert_eq!(v.pop().as_deref(), Some("b"));
    assert_eq!(v.len(), 1);
    assert_eq!(v.pop().as_deref(), Some("a"));
    assert_eq!(v.pop(), None);
}

#[test]
fn ex5_slice_methods_through_deref() {
    let mut v: ex05_stack_vec::StackVec<i32, 8> = ex05_stack_vec::StackVec::new();
    for x in [3, 1, 2] {
        v.push(x).unwrap();
    }
    v.sort();
    assert_eq!(&*v, &[1, 2, 3]);
    v.as_mut_slice()[0] = 10;
    assert_eq!(v.iter().sum::<i32>(), 15);
}

#[test]
fn ex5_drops_exactly_the_live_elements() {
    struct Tracked(Rc<Cell<u32>>);
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    {
        let mut v: ex05_stack_vec::StackVec<Tracked, 4> = ex05_stack_vec::StackVec::new();
        for _ in 0..3 {
            assert!(v.push(Tracked(drops.clone())).is_ok());
        }
        let popped = v.pop();
        assert_eq!(
            drops.get(),
            0,
            "popping moves the value out; nothing is dropped yet"
        );
        drop(popped);
        assert_eq!(drops.get(), 1);
    }
    assert_eq!(
        drops.get(),
        3,
        "the two remaining elements dropped once each -- not 2, not 4"
    );
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_counting_allocator_counts() {
    use ex06_allocator::CountingAllocator;
    use std::alloc::{GlobalAlloc, Layout};
    let a = CountingAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();
    // SAFETY: layout has non-zero size; the pointer is freed with the same layout.
    unsafe {
        let p = a.alloc(layout);
        assert!(!p.is_null());
        assert_eq!((a.allocations(), a.bytes_in_use()), (1, 64));
        p.write_bytes(0xAB, 64); // the memory is really ours
        let p = a.realloc(p, layout, 128);
        assert!(!p.is_null());
        assert_eq!(*p, 0xAB, "realloc keeps the contents");
        assert_eq!((a.reallocations(), a.bytes_in_use()), (1, 128));
        a.dealloc(p, Layout::from_size_align(128, 8).unwrap());
    }
    assert_eq!((a.deallocations(), a.bytes_in_use()), (1, 0));
}

#[test]
fn ex6_arena_strings_stay_valid() {
    let arena = ex06_allocator::Arena::new(16);
    let first = arena.alloc_str("hello");
    let first_ptr = first.as_ptr();
    let others: Vec<&str> = (0..50)
        .map(|i| arena.alloc_str(&format!("word{i}")))
        .collect();
    let big = arena.alloc_str(&"z".repeat(100));
    assert_eq!(first, "hello", "earlier strings survive later allocations");
    assert_eq!(first.as_ptr(), first_ptr, "...without moving");
    assert_eq!(others[49], "word49");
    assert_eq!(big.len(), 100);
    assert!(arena.chunk_count() > 1);
    assert_eq!(arena.alloc_str(""), "");
    assert_eq!(arena.alloc_str("héllo ✓"), "héllo ✓");
}

// ---- Exercise 7 --------------------------------------------------------------

#[cfg(unix)]
#[test]
fn ex7_posix() {
    assert_eq!(ex07_system::process_id(), std::process::id());
    let host = ex07_system::hostname().unwrap();
    assert!(!host.is_empty());
    assert!(!host.contains('\0'));
}
