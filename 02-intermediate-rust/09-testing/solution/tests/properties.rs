//! Property-based tests: proptest generates hundreds of inputs per property
//! and, on failure, shrinks to the smallest input that still fails.

use m09_testing_solution::roman::{from_roman, to_roman};
use m09_testing_solution::slug::slugify;
use proptest::prelude::*;

proptest! {
    #[test]
    fn roman_roundtrip(n in 1u32..=3999) {
        let numeral = to_roman(n).unwrap();
        prop_assert_eq!(from_roman(&numeral), Ok(n));
    }

    /// The roundtrip can't catch a to_roman that writes 90 as LXXXX, because
    /// from_roman accepts that too. This property can: in canonical numerals
    /// no symbol repeats four times. (Before the fix, proptest shrinks the
    /// failure down to n = 90.)
    #[test]
    fn roman_is_canonical(n in 1u32..=3999) {
        let numeral = to_roman(n).unwrap();
        for symbol in ['I', 'X', 'C', 'M'] {
            let four: String = std::iter::repeat_n(symbol, 4).collect();
            prop_assert!(!numeral.contains(&four), "{} -> {}", n, numeral);
        }
        for symbol in ['V', 'L', 'D'] {
            prop_assert!(numeral.matches(symbol).count() <= 1, "{} -> {}", n, numeral);
        }
    }

    #[test]
    fn slug_has_only_allowed_characters(title in ".*") {
        let slug = slugify(&title);
        prop_assert!(slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{:?}", slug);
    }

    #[test]
    fn slug_has_no_stray_hyphens(title in "[a-zA-Z0-9 ,.!-]{0,40}") {
        let slug = slugify(&title);
        prop_assert!(!slug.starts_with('-') && !slug.ends_with('-'), "{:?}", slug);
        prop_assert!(!slug.contains("--"), "{:?} -> {:?}", title, slug);
    }

    #[test]
    fn slugify_is_idempotent(title in ".*") {
        let once = slugify(&title);
        prop_assert_eq!(slugify(&once), once.clone());
    }
}
