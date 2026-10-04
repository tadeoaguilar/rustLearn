use crate::sut::*;

// ---- Exercise 1 --------------------------------------------------------------

#[test]
fn ex1_dedup() {
    use ex01_vectors::*;
    assert_eq!(dedup_keep_order(&[3, 1, 3, 2, 1, 4]), vec![3, 1, 2, 4]);
    assert_eq!(dedup_keep_order(&["a", "a"]), vec!["a"]);
    assert_eq!(dedup_keep_order::<i32>(&[]), Vec::<i32>::new());
    assert_eq!(dedup_sorted(vec![3, 1, 3, 2, 1, 4]), vec![1, 2, 3, 4]);
}

#[test]
fn ex1_split() {
    use ex01_vectors::*;
    assert_eq!(
        split_on(&[1, 0, 2, 3, 0, 4], &0),
        vec![vec![1], vec![2, 3], vec![4]]
    );
    assert_eq!(
        split_on(&[0, 1], &0),
        vec![vec![], vec![1]],
        "leading separator gives an empty first part"
    );
    assert_eq!(
        split_at_value(&[1, 2, 3, 4], &3),
        Some((vec![1, 2], vec![4]))
    );
    assert_eq!(split_at_value(&[1, 2], &9), None);
}

#[test]
fn ex1_merge_sorted() {
    use ex01_vectors::merge_sorted;
    assert_eq!(
        merge_sorted(&[1, 4, 9], &[2, 3, 10]),
        vec![1, 2, 3, 4, 9, 10]
    );
    assert_eq!(merge_sorted(&[], &[1, 2]), vec![1, 2]);
    assert_eq!(merge_sorted(&[1, 1], &[1]), vec![1, 1, 1]);
}

// ---- Exercise 2 --------------------------------------------------------------

#[test]
fn ex2_word_frequency() {
    use ex02_hashmaps::*;
    let counts = word_frequency("The cat and the hat. THE end!");
    assert_eq!(
        counts.get("the"),
        Some(&3),
        "case-insensitive, punctuation stripped"
    );
    assert_eq!(counts.get("cat"), Some(&1));
    assert_eq!(counts.get("hat."), None);
    assert_eq!(
        top_words("b a b c a b", 2),
        vec![("b".to_string(), 3), ("a".to_string(), 2)]
    );
    assert_eq!(
        top_words("z y x", 2),
        vec![("x".to_string(), 1), ("y".to_string(), 1)],
        "ties alphabetical"
    );
}

#[test]
fn ex2_group_by_age() {
    use ex02_hashmaps::*;
    let people = vec![
        Person {
            name: "Ann".into(),
            age: 30,
        },
        Person {
            name: "Bob".into(),
            age: 25,
        },
        Person {
            name: "Cat".into(),
            age: 30,
        },
    ];
    let groups = group_by_age(&people);
    assert_eq!(
        groups.get(&30),
        Some(&vec!["Ann".to_string(), "Cat".to_string()])
    );
    assert_eq!(groups.get(&25), Some(&vec!["Bob".to_string()]));
    assert_eq!(groups.keys().copied().collect::<Vec<_>>(), vec![25, 30]);
}

#[test]
fn ex2_lru_cache_evicts_least_recently_used() {
    use ex02_hashmaps::LruCache;
    let mut cache = LruCache::new(2);
    assert_eq!(cache.set("a", 1), None);
    assert_eq!(cache.set("b", 2), None);
    assert_eq!(cache.get(&"a"), Some(&1)); // a is now most recent
    assert_eq!(
        cache.set("c", 3),
        Some(("b", 2)),
        "b was least recently used"
    );
    assert_eq!(cache.get(&"b"), None);
    assert_eq!(cache.len(), 2);
    assert_eq!(
        cache.set("a", 10),
        None,
        "replacing an existing key evicts nothing"
    );
    assert_eq!(cache.get(&"a"), Some(&10));
    assert_eq!(cache.evict(&"a"), Some(10));
    assert_eq!(cache.len(), 1);
}

// ---- Exercise 3 --------------------------------------------------------------

#[test]
fn ex3_basics() {
    let b = ex03_iterator_basics::basics(&[1, 2, 3, 4, 5]);
    assert_eq!(b.sum, 15);
    assert_eq!(b.product, 120);
    assert_eq!(b.doubled, vec![2, 4, 6, 8, 10]);
    assert_eq!(b.evens, vec![2, 4]);
    assert_eq!(b.even_squares, vec![4, 16]);
    assert_eq!(b.first_three, vec![1, 2, 3]);
    assert_eq!(b.skip_two, vec![3, 4, 5]);
}

#[test]
fn ex3_tasks() {
    use ex03_iterator_basics::*;
    assert_eq!(sum_of_squares_of_evens(&[1, 2, 3, 4, 5, 6]), 4 + 16 + 36);
    assert_eq!(first_divisible_by_3_and_5(&[3, 5, 9, 30, 45]), Some(30));
    assert_eq!(first_divisible_by_3_and_5(&[1, 2]), None);
    assert_eq!(first_positive_divisible_by(3, 5), 15);
    assert_eq!(flatten(&[vec![1, 2], vec![], vec![3]]), vec![1, 2, 3]);
}

// ---- Exercise 4 --------------------------------------------------------------

#[test]
fn ex4_basics() {
    let b = ex04_advanced_iterators::advanced_basics();
    assert_eq!(b.sum, 15);
    assert_eq!(b.running_sum, vec![1, 3, 6, 10, 15]);
    assert_eq!(b.combined, vec![(1, "one"), (2, "two"), (3, "three")]);
    assert_eq!(b.flat, vec![1, 2, 3, 4]);
}

#[test]
fn ex4_fibonacci_ends_at_the_largest_u64_fibonacci() {
    use ex04_advanced_iterators::fibonacci;
    assert_eq!(
        fibonacci().take(10).collect::<Vec<_>>(),
        vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
    );
    assert_eq!(fibonacci().count(), 94, "F0 through F93");
    assert_eq!(fibonacci().last(), Some(12_200_160_415_121_876_738));
}

#[test]
fn ex4_cartesian_and_partition() {
    use ex04_advanced_iterators::*;
    assert_eq!(
        cartesian_product(&[1, 2], &['a', 'b']),
        vec![(1, 'a'), (1, 'b'), (2, 'a'), (2, 'b')]
    );
    assert_eq!(cartesian_product::<i32, i32>(&[], &[1]), vec![]);
    assert_eq!(
        partition_even_odd(&[1, 2, 3, 4, 5, 6]),
        (vec![2, 4, 6], vec![1, 3, 5])
    );
}

// ---- Exercise 5 --------------------------------------------------------------

#[test]
fn ex5_fibonacci_struct_does_not_overflow() {
    use ex05_custom_iterators::Fibonacci;
    assert_eq!(
        Fibonacci::new().take(10).collect::<Vec<_>>(),
        vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
    );
    // The exercise's version panics here in a debug build.
    assert_eq!(Fibonacci::new().count(), 94);
}

#[test]
fn ex5_step_range() {
    use ex05_custom_iterators::StepRange;
    assert_eq!(
        StepRange::new(0, 10, 3).collect::<Vec<_>>(),
        vec![0, 3, 6, 9]
    );
    assert_eq!(
        StepRange::new(10, 0, -4).collect::<Vec<_>>(),
        vec![10, 6, 2]
    );
    assert_eq!(StepRange::new(5, 5, 1).count(), 0);
    assert_eq!(
        StepRange::new(0, 10, -1).count(),
        0,
        "counting down from below the end yields nothing"
    );
}

#[test]
#[should_panic]
fn ex5_step_range_rejects_zero_step() {
    ex05_custom_iterators::StepRange::new(0, 10, 0);
}

#[test]
fn ex5_cycle_slice() {
    use ex05_custom_iterators::CycleSlice;
    let colors = ["r", "g", "b"];
    assert_eq!(
        CycleSlice::new(&colors)
            .take(7)
            .copied()
            .collect::<String>(),
        "rgbrgbr"
    );
    assert_eq!(
        CycleSlice::<i32>::new(&[]).next(),
        None,
        "empty slice must not panic"
    );
}

#[test]
fn ex5_primes() {
    use ex05_custom_iterators::Primes;
    assert_eq!(
        Primes::new().take(10).collect::<Vec<_>>(),
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
    );
    assert_eq!(Primes::new().nth(999), Some(7919));
}

// ---- Exercise 6 --------------------------------------------------------------

#[test]
fn ex6_apply_repeat_call_once() {
    use ex06_closures::*;
    let x = 5;
    assert_eq!(apply(|y| x + y, 3), 8);
    let mut n = 0;
    repeat(|| n += 2, 4);
    assert_eq!(n, 8);
    let owned = String::from("moved");
    assert_eq!(call_once(move || owned), "moved");
}

#[test]
fn ex6_counters_are_independent() {
    let mut a = ex06_closures::make_counter();
    let mut b = ex06_closures::make_counter();
    assert_eq!((a(), a(), a()), (1, 2, 3));
    assert_eq!(b(), 1);
}

#[test]
fn ex6_compose_and_pipeline() {
    use ex06_closures::*;
    let f = compose(|x: i32| x + 1, |x| x * 2);
    assert_eq!(f(5), 12, "f then g: (5 + 1) * 2");
    let to_len = compose(|s: &str| s.trim(), |s: &str| s.len());
    assert_eq!(to_len("  abc "), 3);
    let p = Pipeline::new()
        .then(|x: i32| x + 1)
        .then(|x| x * 10)
        .then(|x| x - 3);
    assert_eq!(p.run(4), 47);
    assert_eq!(
        Pipeline::<i32>::new().run(7),
        7,
        "an empty pipeline is the identity"
    );
}

// ---- Bonus -------------------------------------------------------------------

#[test]
fn bonus_revenue_and_top() {
    use bonus_pipeline::*;
    let sales = sample_sales();
    let revenue = revenue_by_product(&sales);
    assert_eq!(revenue["Widget"], 70.0);
    assert_eq!(revenue["Gadget"], 45.0 + 126.0);
    assert_eq!(
        top_products(&sales, 2),
        vec![("Gadget".to_string(), 171.0), ("Gizmo".to_string(), 120.0)]
    );
}

#[test]
fn bonus_top_products_survives_nan() {
    use bonus_pipeline::*;
    let sales = vec![Sale::new("A", f64::NAN, 1), Sale::new("B", 1.0, 1)];
    assert_eq!(
        top_products(&sales, 5).len(),
        2,
        "partial_cmp().unwrap() would panic here"
    );
}

#[test]
fn bonus_tasks() {
    use bonus_pipeline::*;
    let sales = sample_sales();
    let above: Vec<&str> = sales_above(&sales, 60.0)
        .iter()
        .map(|s| s.product.as_str())
        .collect();
    assert_eq!(above, vec!["Gizmo", "Gadget"]);

    let avg = average_price_per_product(&sales);
    assert_eq!(avg["Widget"], 10.0);
    assert_eq!(avg["Gadget"], 171.0 / 12.0, "weighted by quantity");

    assert_eq!(
        products_with_quantity_over(&sales, 10),
        vec!["Doohickey", "Gadget"]
    );

    let groups = group_by_price_range(&sales);
    assert_eq!(groups[&PriceRange::Budget].len(), 1);
    assert_eq!(groups[&PriceRange::Mid].len(), 4);
    assert_eq!(groups[&PriceRange::Premium].len(), 1);
    assert_eq!(
        PriceRange::of(5.0),
        PriceRange::Mid,
        "boundaries: 5 is Mid, 50 is Premium"
    );
    assert_eq!(PriceRange::of(50.0), PriceRange::Premium);
}
