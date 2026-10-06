use std::alloc::{GlobalAlloc, Layout};
use std::mem::{align_of, offset_of, size_of};
use std::num::NonZeroU32;

use crate::sut::bonus_arena_tree::{NodeId, Tree};
use crate::sut::ex01_bump::{self as bump, Bump, GlobalBump};
use crate::sut::ex02_pool::{ObjectPool, Slab};
use crate::sut::ex03_counting::{self as counting, Counting};
use crate::sut::ex04_layout::{self as layout, Packed, Padded};
use crate::sut::ex05_soa as soa;

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_align_up() {
    assert_eq!(bump::align_up(0, 8), Some(0));
    assert_eq!(bump::align_up(1, 8), Some(8));
    assert_eq!(bump::align_up(8, 8), Some(8));
    assert_eq!(bump::align_up(13, 4), Some(16));
    assert_eq!(bump::align_up(usize::MAX, 8), None);
}

#[test]
fn ex1_bump_allocates_aligned_and_in_order() {
    let arena = Bump::new(256);
    assert_eq!(
        (arena.capacity(), arena.used(), arena.remaining()),
        (256, 0, 256)
    );
    let a = arena.alloc(Layout::new::<u8>()).unwrap();
    let b = arena.alloc(Layout::new::<u64>()).unwrap();
    assert_eq!(b.as_ptr() as usize % 8, 0, "aligned");
    assert!(
        b.as_ptr() as usize > a.as_ptr() as usize,
        "the offset only moves forward"
    );
    let big = arena
        .alloc(Layout::from_size_align(64, 64).unwrap())
        .unwrap();
    assert_eq!(big.as_ptr() as usize % 64, 0);
    assert!(arena.used() <= 256 && arena.remaining() == 256 - arena.used());
    assert!(
        arena
            .alloc(Layout::from_size_align(1000, 1).unwrap())
            .is_none(),
        "doesn't fit"
    );
}

#[test]
fn ex1_typed_allocations_and_reset() {
    let mut arena = Bump::new(128);
    let x = arena.alloc_value(41u32).unwrap();
    let y = arena.alloc_value(2.5f64).unwrap();
    *x += 1;
    assert_eq!(
        (*x, *y),
        (42, 2.5),
        "both stay valid: allocations never overlap"
    );
    let s = arena.alloc_str("héllo").unwrap();
    s.make_ascii_uppercase();
    assert_eq!(s, "HéLLO");
    let nums = arena.alloc_slice(&[1u16, 2, 3]).unwrap();
    nums[0] = 9;
    assert_eq!(nums, [9, 2, 3]);
    let used = arena.used();
    assert!(used >= 4 + 8 + 6 + 6);
    arena.reset();
    assert_eq!(arena.used(), 0);
    assert!(
        arena.alloc_slice(&[0u8; 128]).is_some(),
        "the whole buffer is free again"
    );
    assert!(arena.alloc_value(1u8).is_none());
}

#[test]
fn ex1_global_bump() {
    let global: Box<GlobalBump<4096>> = Box::new(GlobalBump::new());
    let layout = Layout::from_size_align(100, 16).unwrap();
    // SAFETY: `layout` has a non-zero size; the pointers aren't used after `global` drops.
    let (a, b) = unsafe { (global.alloc(layout), global.alloc(layout)) };
    assert!(!a.is_null() && !b.is_null());
    assert_eq!((a as usize % 16, b as usize % 16), (0, 0));
    assert!(b as usize >= a as usize + 100, "no overlap");
    // SAFETY: as above.
    unsafe { global.dealloc(a, layout) }; // a no-op
    let too_big = Layout::from_size_align(5000, 8).unwrap();
    // SAFETY: as above.
    assert!(
        unsafe { global.alloc(too_big) }.is_null(),
        "out of memory is null, not a panic"
    );
    // threads claim disjoint ranges
    let global: &'static GlobalBump<4096> = Box::leak(global);
    let handles: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(move || {
                // SAFETY: as above (the leaked allocator lives forever).
                (0..10)
                    .map(|_| unsafe { global.alloc(Layout::new::<u64>()) } as usize)
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let mut all: Vec<usize> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();
    all.sort();
    all.dedup();
    assert_eq!(all.len(), 80, "every allocation distinct");
}

// ---------------------------------------------------------------- Exercise 2

#[test]
fn ex2_slab_reuses_slots_with_new_generations() {
    let mut slab = Slab::new();
    assert!(slab.is_empty());
    let a = slab.insert("a");
    let b = slab.insert("b");
    assert_eq!((slab.len(), slab.capacity()), (2, 2));
    assert_eq!(slab.remove(a), Some("a"));
    assert_eq!(slab.remove(a), None, "already removed");
    let c = slab.insert("c");
    assert_eq!(
        (c.index, c.generation),
        (a.index, a.generation + 1),
        "reused the slot"
    );
    assert_eq!(slab.capacity(), 2, "no growth");
    assert_eq!(
        slab.get(a),
        None,
        "the stale handle doesn't see the new value"
    );
    assert_eq!(slab.get(c), Some(&"c"));
    *slab.get_mut(b).unwrap() = "B";
    assert_eq!(slab.get(b), Some(&"B"));
    assert_eq!(slab.get_mut(a), None);
    let live: Vec<_> = slab.iter().map(|(h, v)| (h.index, *v)).collect();
    assert_eq!(live, [(0, "c"), (1, "B")]);
}

#[test]
fn ex2_slab_free_list_is_lifo() {
    let mut slab = Slab::new();
    let handles: Vec<_> = (0..5).map(|i| slab.insert(i)).collect();
    slab.remove(handles[1]);
    slab.remove(handles[3]);
    assert_eq!(slab.insert(10).index, 3, "most recently freed first");
    assert_eq!(slab.insert(11).index, 1);
    assert_eq!(slab.insert(12).index, 5, "then grow");
    assert_eq!(slab.len(), 6);
}

#[test]
fn ex2_object_pool_recycles() {
    let pool: ObjectPool<Vec<u8>> = ObjectPool::new(|| Vec::with_capacity(64), Vec::clear);
    {
        let mut a = pool.take();
        a.extend_from_slice(b"hello");
        let mut b = pool.take();
        b.push(1);
        assert_eq!((pool.created(), pool.available()), (2, 0));
    }
    assert_eq!(pool.available(), 2, "both went back on drop");
    for _ in 0..100 {
        let buf = pool.take();
        assert!(buf.is_empty(), "reset before reuse");
        assert!(buf.capacity() >= 64, "the allocation was kept");
    }
    assert_eq!(pool.created(), 2, "no new objects needed");
}

// ---------------------------------------------------------------- Exercise 3

#[test]
fn ex3_counting_wrapper_without_registering() {
    let counting = Counting::system();
    let layout = Layout::from_size_align(256, 8).unwrap();
    // SAFETY: a valid non-zero layout; each block is freed once with its layout.
    unsafe {
        let a = counting.alloc(layout);
        let b = counting.alloc(Layout::new::<u64>());
        let s = counting.snapshot();
        assert_eq!((s.allocations, s.live_bytes, s.peak_bytes), (2, 264, 264));
        counting.dealloc(a, layout);
        let grown = counting.realloc(b, Layout::new::<u64>(), 32);
        let s = counting.snapshot();
        assert_eq!((s.live_bytes, s.peak_bytes), (32, 264));
        counting.dealloc(grown, Layout::from_size_align(32, 8).unwrap());
    }
    let s = counting.snapshot();
    assert_eq!(s.live_bytes, 0);
    assert_eq!(
        s.allocations, s.deallocations,
        "every allocation freed (a realloc counts as one of each)"
    );
}

#[test]
fn ex3_the_functions_agree() {
    let words = ["a", "bb", "ccc"];
    assert_eq!(counting::join_naive(&words), "a,bb,ccc");
    assert_eq!(counting::join_reserved(&words), "a,bb,ccc");
    assert_eq!(counting::join_reserved(&[]), "");
    assert_eq!(
        counting::join_reserved(&words).capacity(),
        8,
        "exactly the right size"
    );
    assert_eq!(
        counting::count_long_words("tiny words and enormous ones", 4),
        2
    );
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_layout_c_matches_the_compiler() {
    let (offsets, size, align) = layout::layout_c(&[(1, 1), (8, 8), (1, 1)]);
    assert_eq!(
        offsets,
        [
            offset_of!(Padded, flag),
            offset_of!(Padded, value),
            offset_of!(Padded, tag)
        ]
    );
    assert_eq!((size, align), (size_of::<Padded>(), align_of::<Padded>()));
    let (offsets, size, _) = layout::layout_c(&[(8, 8), (1, 1), (1, 1)]);
    assert_eq!(
        offsets,
        [
            offset_of!(Packed, value),
            offset_of!(Packed, flag),
            offset_of!(Packed, tag)
        ]
    );
    assert_eq!(size, size_of::<Packed>());
    assert_eq!(layout::layout_c(&[]), (vec![], 0, 1));
    assert_eq!(
        layout::layout_c(&[(2, 2), (4, 4), (1, 1)]),
        (vec![0, 4, 8], 12, 4)
    );
}

#[test]
fn ex4_padding_and_order() {
    assert_eq!(layout::padding(&[(1, 1), (8, 8), (1, 1)]), 14);
    assert_eq!(layout::padding(&[(8, 8), (1, 1), (1, 1)]), 6);
    let fields = [(1, 1), (4, 4), (2, 2), (8, 8)];
    let order = layout::best_order(&fields);
    assert_eq!(order, [3, 1, 2, 0]);
    let reordered: Vec<(usize, usize)> = order.iter().map(|&i| fields[i]).collect();
    assert_eq!(layout::padding(&reordered), 1);
    assert!(layout::padding(&reordered) < layout::padding(&fields));
}

#[test]
fn ex4_niches() {
    assert!(layout::option_is_free::<&u8>());
    assert!(layout::option_is_free::<Box<[u8; 100]>>());
    assert!(layout::option_is_free::<NonZeroU32>());
    assert!(
        layout::option_is_free::<Vec<u8>>(),
        "Vec's pointer is non-null"
    );
    assert!(
        !layout::option_is_free::<u32>(),
        "every u32 bit pattern is a valid u32"
    );
    assert!(
        layout::option_is_free::<bool>(),
        "bool has 254 spare values"
    );
}

// ---------------------------------------------------------------- Exercise 5

#[test]
fn ex5_aos_and_soa_agree() {
    let mut aos = soa::generate(1000);
    assert_eq!(aos.particles.len(), 1000);
    assert_eq!(soa::generate(1000), aos, "deterministic");
    let mut s = aos.to_soa();
    assert_eq!(s.len(), 1000);
    assert_eq!(s.to_aos(), aos, "round trip");
    for _ in 0..5 {
        aos.step(0.1);
        s.step(0.1);
    }
    assert_eq!(s.to_aos(), aos, "same simulation");
    assert_eq!(aos.kinetic_energy(), s.kinetic_energy());
    assert_eq!(aos.mean_x(), s.mean_x());
    let dead = aos.particles.iter().position(|p| !p.alive).unwrap();
    assert_eq!(
        aos.particles[dead].position,
        soa::generate(1000).particles[dead].position,
        "dead particles don't move"
    );
    assert!(soa::ParticlesSoa::default().is_empty());
    assert_eq!(soa::ParticlesAos::default().mean_x(), 0.0);
}

#[test]
fn ex5_row_and_column_order_agree() {
    let (rows, cols) = (37, 53);
    let data: Vec<u32> = (0..rows * cols).map(|i| (i * 7 % 101) as u32).collect();
    let expected: u64 = data.iter().map(|v| *v as u64).sum();
    assert_eq!(soa::sum_row_major(&data, rows, cols), expected);
    assert_eq!(soa::sum_col_major(&data, rows, cols), expected);
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_arena_tree() {
    let mut tree = Tree::new(1u64);
    let root = tree.root();
    let a = tree.add_child(root, 2);
    let b = tree.add_child(root, 3);
    let a1 = tree.add_child(a, 4);
    let a2 = tree.add_child(a, 5);
    let b1 = tree.add_child(b, 6);
    assert_eq!(tree.len(), 6);
    assert_eq!(tree.node(a1).parent, Some(a));
    assert_eq!(tree.node(a).children, [a1, a2]);
    assert_eq!(tree.path_to_root(b1), [b1, b, root]);
    assert_eq!((tree.depth(root), tree.depth(a2)), (0, 2));
    assert_eq!(
        tree.descendants(root),
        [root, a, a1, a2, b, b1],
        "pre-order"
    );
    assert_eq!(tree.subtree_sum(a), 11);
    *tree.value_mut(a2) = 100;
    assert_eq!(tree.subtree_sum(root), 1 + 2 + 3 + 4 + 100 + 6);
    // deep trees don't overflow the stack
    let mut deep = Tree::new(0u64);
    let mut node = deep.root();
    for i in 0..100_000 {
        node = deep.add_child(node, i);
    }
    assert_eq!(deep.depth(node), 100_000);
    assert_eq!(deep.descendants(NodeId(0)).len(), 100_001);
}
