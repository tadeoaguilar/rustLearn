use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

use crate::sut::ex03_cache::{self as cache, Matrix};
use crate::sut::{bonus_swar as swar, ex01_hotpath as hot, ex02_simd as simd, ex04_branches as br};

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_fnv1a_matches_reference_values() {
    let hash = |bytes: &[u8]| {
        let mut h = hot::Fnv1a::default();
        h.write(bytes);
        h.finish()
    };
    assert_eq!(hash(b""), 0xcbf2_9ce4_8422_2325, "the offset basis");
    assert_eq!(hash(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(hash(b"foobar"), 0x8594_4171_f739_67e8);
    let build = hot::FnvBuildHasher::default();
    assert_eq!(build.hash_one("same"), build.hash_one("same"));
    let mut map: HashMap<&str, i32, hot::FnvBuildHasher> = HashMap::default();
    map.insert("k", 1);
    assert_eq!(map["k"], 1);
}

#[test]
fn ex1_fast_matches_naive() {
    for text in [
        "",
        "one",
        "The the THE tHe",
        "Don't stop -- don't! (it's fine) it's",
        "ünïcödé separates: naïve café",
        "a b c d e f g h a b c a",
        "tie tie zed zed alpha alpha",
    ] {
        for n in [0, 1, 2, 3, 10] {
            assert_eq!(
                hot::top_words_fast(text, n),
                hot::top_words_naive(text, n),
                "{text:?}, n = {n}"
            );
        }
    }
    let big = hot::sample_text(50_000);
    assert_eq!(hot::top_words_fast(&big, 7), hot::top_words_naive(&big, 7));
    assert_eq!(
        hot::top_words_fast(&big, 10_000),
        hot::top_words_naive(&big, 10_000),
        "n larger than the vocabulary"
    );
}

#[test]
fn ex1_ties_break_alphabetically() {
    let top = hot::top_words_fast("b a c b a c d", 3);
    assert_eq!(
        top,
        [
            ("a".to_string(), 2),
            ("b".to_string(), 2),
            ("c".to_string(), 2)
        ]
    );
}

// ---------------------------------------------------------------- Exercise 2

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-3 * a.abs().max(b.abs()).max(1.0)
}

#[test]
fn ex2_dot_products_agree() {
    for n in [0usize, 1, 3, 7, 8, 15, 16, 17, 31, 33, 1000, 4099] {
        let a: Vec<f32> = (0..n).map(|i| ((i * 37) % 11) as f32 - 5.0).collect();
        let b: Vec<f32> = (0..n).map(|i| ((i * 13) % 7) as f32 * 0.5).collect();
        let expected = simd::dot_scalar(&a, &b);
        assert!(
            close(simd::dot_unrolled(&a, &b), expected),
            "unrolled, n = {n}"
        );
        assert!(
            close(simd::dot_simd(&a, &b), expected),
            "simd ({}), n = {n}",
            simd::simd_level()
        );
    }
    assert_eq!(simd::dot_simd(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]), 32.0);
}

#[test]
#[should_panic]
fn ex2_mismatched_lengths_panic() {
    simd::dot_simd(&[1.0], &[1.0, 2.0]);
}

#[test]
fn ex2_byte_counts_are_exact() {
    let hay: Vec<u8> = (0..10_007u32).map(|i| (i * 7 % 251) as u8).collect();
    for needle in [0u8, 7, 128, 250, 255] {
        let expected = simd::count_byte_scalar(&hay, needle);
        assert_eq!(
            simd::count_byte_simd(&hay, needle),
            expected,
            "needle {needle}"
        );
        for len in [0, 1, 15, 16, 17, 100] {
            assert_eq!(
                simd::count_byte_simd(&hay[..len], needle),
                simd::count_byte_scalar(&hay[..len], needle)
            );
        }
    }
    assert_eq!(
        simd::count_byte_simd(&[0xFF; 64], 0xFF),
        64,
        "the sign bit doesn't confuse the comparison"
    );
    let level = simd::simd_level();
    assert!(["avx2+fma", "sse2", "portable"].contains(&level), "{level}");
}

// ---------------------------------------------------------------- Exercise 3

#[test]
fn ex3_every_order_gives_the_same_product() {
    for n in [1usize, 2, 7, 16, 33, 64] {
        let (a, b) = (Matrix::sample(n, 1), Matrix::sample(n, 2));
        let expected = cache::multiply_naive(&a, &b);
        assert_eq!(cache::multiply_ikj(&a, &b), expected, "ikj, n = {n}");
        assert_eq!(
            cache::multiply_transposed(&a, &b),
            expected,
            "transposed, n = {n}"
        );
        for tile in [1, 4, 8, 13, 64, 100] {
            assert_eq!(
                cache::multiply_blocked(&a, &b, tile),
                expected,
                "tile {tile}, n = {n}"
            );
        }
    }
}

#[test]
fn ex3_naive_is_right_and_transpose_works() {
    let a = Matrix {
        n: 2,
        data: vec![1, 2, 3, 4],
    };
    let b = Matrix {
        n: 2,
        data: vec![5, 6, 7, 8],
    };
    assert_eq!(cache::multiply_naive(&a, &b).data, [19, 22, 43, 50]);
    assert_eq!(a.transposed().data, [1, 3, 2, 4]);
    assert_eq!(a.transposed().transposed(), a);
    let identity = Matrix {
        n: 3,
        data: vec![1, 0, 0, 0, 1, 0, 0, 0, 1],
    };
    let m = Matrix::sample(3, 5);
    assert_eq!(cache::multiply_blocked(&m, &identity, 2), m);
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_counting_agrees() {
    let values = br::random_values(10_001, 1000, 3);
    for t in [0, 1, 500, 999, 1000, 5000] {
        assert_eq!(
            br::count_below_branchless(&values, t),
            br::count_below_branchy(&values, t),
            "threshold {t}"
        );
    }
    assert_eq!(br::count_below_branchy(&[1, 5, 3, 9], 4), 2);
}

#[test]
fn ex4_lower_bound_matches_partition_point() {
    let mut sorted = br::random_values(1000, 300, 9);
    sorted.sort_unstable();
    for target in -1..=301 {
        assert_eq!(
            br::lower_bound_branchless(&sorted, target),
            sorted.partition_point(|v| *v < target),
            "target {target}"
        );
    }
    assert_eq!(br::lower_bound_branchless(&[], 5), 0);
    assert_eq!(br::lower_bound_branchless(&[5], 5), 0);
    assert_eq!(br::lower_bound_branchless(&[5], 6), 1);
    assert_eq!(
        br::lower_bound_branchless(&[1, 2, 2, 2, 3], 2),
        1,
        "the first of equal elements"
    );
}

#[test]
fn ex4_sums_agree() {
    for n in [0usize, 1, 3, 4, 5, 1001] {
        let v: Vec<u64> = (0..n as u64).map(|i| i * i).collect();
        assert_eq!(br::sum_iter(&v), br::sum_indexed(&v), "n = {n}");
    }
    assert_eq!(br::sum_iter(&[u64::MAX, 2]), 1, "wrapping");
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_zero_bytes() {
    assert!(!swar::has_zero_byte(0x0101_0101_0101_0101));
    assert!(swar::has_zero_byte(0x0101_0100_0101_0101));
    assert!(swar::has_zero_byte(0));
    assert!(!swar::has_zero_byte(u64::MAX));
    assert_eq!(swar::splat(0xAB), 0xABAB_ABAB_ABAB_ABAB);
    let word = u64::from_le_bytes([1, 2, 0, 4, 5, 6, 7, 8]);
    assert_eq!(
        swar::zero_byte_mask(word).trailing_zeros() / 8,
        2,
        "the lowest set bit marks the first zero"
    );
}

#[test]
fn bonus_find_and_count() {
    let text = b"the quick brown fox jumps over the lazy dog";
    for needle in *b"tqzg !" {
        assert_eq!(
            swar::find_byte(text, needle),
            text.iter().position(|&b| b == needle),
            "{}",
            needle as char
        );
        assert_eq!(
            swar::count_byte(text, needle),
            text.iter().filter(|&&b| b == needle).count(),
            "{}",
            needle as char
        );
    }
    let tricky: Vec<u8> = (0..=255u8).chain(0..=255u8).collect();
    for needle in [0u8, 1, 127, 128, 129, 255] {
        assert_eq!(
            swar::count_byte(&tricky, needle),
            2,
            "needle {needle}: no false positives"
        );
        assert_eq!(swar::find_byte(&tricky, needle), Some(needle as usize));
    }
    assert_eq!(swar::find_byte(b"", b'a'), None);
}
