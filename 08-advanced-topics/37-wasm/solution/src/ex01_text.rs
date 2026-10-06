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
    text.split_whitespace().count() as u32
}

/// A URL slug: lowercase ASCII letters and digits, runs of anything else
/// collapsed to one `-`, no leading or trailing `-`.
/// `slugify("Hello, WASM World!") == "hello-wasm-world"`.
#[wasm_bindgen]
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut pending_dash = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(c.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    slug
}

/// Escape `& < > " '` for HTML.
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Inline markup on already-escaped text: `**bold**`, `*em*`, `` `code` ``.
fn inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(['*', '`']) {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        let (marker, tag) = if tail.starts_with("**") {
            ("**", "strong")
        } else if tail.starts_with('*') {
            ("*", "em")
        } else {
            ("`", "code")
        };
        match tail[marker.len()..].find(marker) {
            Some(end) if end > 0 => {
                let inner = &tail[marker.len()..marker.len() + end];
                let inner = if tag == "code" {
                    inner.to_string()
                } else {
                    inline(inner)
                };
                out.push_str(&format!("<{tag}>{inner}</{tag}>"));
                rest = &tail[marker.len() * 2 + end..];
            }
            _ => {
                out.push_str(marker);
                rest = &tail[marker.len()..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// A small Markdown subset to HTML: `#`..`######` headings, `- ` list
/// items (consecutive ones form a `<ul>`), paragraphs (lines separated by
/// blank lines), and inline `**bold**`, `*em*`, `` `code` ``. Everything is
/// HTML-escaped first, so no input can inject markup.
#[wasm_bindgen]
pub fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut in_list = false;
    let flush_paragraph = |html: &mut String, paragraph: &mut Vec<String>| {
        if !paragraph.is_empty() {
            html.push_str(&format!("<p>{}</p>\n", inline(&paragraph.join(" "))));
            paragraph.clear();
        }
    };
    for line in markdown.lines() {
        let line = escape_html(line.trim_end());
        let hashes = line.chars().take_while(|c| *c == '#').count();
        let is_heading = (1..=6).contains(&hashes) && line[hashes..].starts_with(' ');
        let item = line.strip_prefix("- ");
        if item.is_none() && in_list {
            html.push_str("</ul>\n");
            in_list = false;
        }
        if is_heading {
            flush_paragraph(&mut html, &mut paragraph);
            html.push_str(&format!(
                "<h{hashes}>{}</h{hashes}>\n",
                inline(line[hashes..].trim())
            ));
        } else if let Some(item) = item {
            flush_paragraph(&mut html, &mut paragraph);
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>\n", inline(item.trim())));
        } else if line.trim().is_empty() {
            flush_paragraph(&mut html, &mut paragraph);
        } else {
            paragraph.push(line.trim().to_string());
        }
    }
    flush_paragraph(&mut html, &mut paragraph);
    if in_list {
        html.push_str("</ul>\n");
    }
    html
}
