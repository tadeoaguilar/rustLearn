//! Exercise 1: a WebAssembly library for JavaScript -- text utilities.
//!
//! `#[wasm_bindgen]` on a function exports it to JavaScript: wasm-pack
//! generates the glue that copies a JS string into WASM memory, calls the
//! function, and copies the result back. The functions themselves are
//! ordinary Rust -- compiled natively they're just functions, which is how
//! this module tests them without a browser.

use wasm_bindgen::prelude::*;

/// Words (runs of non-whitespace).
#[wasm_bindgen]
pub fn word_count(text: &str) -> u32 {
    todo!("Exercise 1")
}

/// A URL slug: lowercase ASCII letters and digits, runs of anything else
/// collapsed to one `-`, no leading or trailing `-`.
/// `slugify("Hello, WASM World!") == "hello-wasm-world"`.
#[wasm_bindgen]
pub fn slugify(text: &str) -> String {
    todo!("Exercise 1")
}

/// Escape `& < > " '` for HTML.
pub fn escape_html(text: &str) -> String {
    todo!("Exercise 1")
}

/// Inline markup on already-escaped text: `**bold**`, `*em*`, `` `code` ``.
fn inline(text: &str) -> String {
    todo!("Exercise 1")
}

/// A small Markdown subset to HTML: `#`..`######` headings, `- ` list
/// items (consecutive ones form a `<ul>`), paragraphs (lines separated by
/// blank lines), and inline `**bold**`, `*em*`, `` `code` ``. Everything is
/// HTML-escaped first, so no input can inject markup.
#[wasm_bindgen]
pub fn markdown_to_html(markdown: &str) -> String {
    todo!("Exercise 1")
}
