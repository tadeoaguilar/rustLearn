//! URL slugs.

/// Anything that isn't an ASCII letter or digit separates words.
/// Private: integration tests can't see it, unit tests below can.
fn is_separator(c: char) -> bool {
    !c.is_ascii_alphanumeric()
}

/// Turns a title into a URL slug: lowercase ASCII letters and digits, words
/// joined by single hyphens, no hyphen at either end.
///
/// ```
/// use m09_testing_solution::slug::slugify;
/// assert_eq!(slugify("Hello, World!"), "hello-world");
/// assert_eq!(slugify("  Rust   2024  edition "), "rust-2024-edition");
/// assert_eq!(slugify("a  b"), "a-b"); // runs of separators collapse
/// assert_eq!(slugify("!!!"), "");
/// ```
///
/// Non-ASCII letters are treated as separators, so `"Café"` becomes `"caf"`.
/// A production slugifier would transliterate first (the `deunicode` crate).
pub fn slugify(title: &str) -> String {
    // BUG FIXED: the original replaced every separator with '-', so
    // "a  b" became "a--b". Splitting on separators and dropping the empty
    // pieces collapses runs and trims both ends in one go.
    title
        .split(is_separator)
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_helper_is_testable_from_unit_tests() {
        assert!(is_separator(' '));
        assert!(is_separator('-'));
        assert!(is_separator('é'));
        assert!(!is_separator('a'));
        assert!(!is_separator('7'));
    }

    #[test]
    fn basic_titles() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Already-a-slug"), "already-a-slug");
        assert_eq!(slugify("C++ & Rust: 2 Languages"), "c-rust-2-languages");
    }

    #[test]
    fn separators_collapse_and_ends_are_trimmed() {
        // Bug #6.
        assert_eq!(slugify("a  b"), "a-b");
        assert_eq!(slugify("--a--b--"), "a-b");
        assert_eq!(slugify(""), "");
    }
}
