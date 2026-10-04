//! URL slugs.

/// Anything that isn't an ASCII letter or digit separates words.
/// Private: integration tests can't see it, unit tests below can.
fn is_separator(c: char) -> bool {
    !c.is_ascii_alphanumeric()
}

/// Turns a title into a URL slug: lowercase ASCII letters and digits, words
/// joined by single hyphens, no hyphen at either end.
///
/// Non-ASCII letters are treated as separators, so `"Café"` becomes `"caf"`.
/// A production slugifier would transliterate first (the `deunicode` crate).
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.trim().chars() {
        if is_separator(c) {
            out.push('-');
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 3: your tests here.
}
