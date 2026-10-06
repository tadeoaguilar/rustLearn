//! Helper attributes, like `#[json(rename = "x", skip)]` (provided).

use syn::{Attribute, LitStr};

/// The options found in `#[<name>(...)]` attributes on an item or field.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Options {
    /// `key = "value"` pairs.
    pub values: Vec<(String, String)>,
    /// Bare words, like `skip` or `primary_key`.
    pub flags: Vec<String>,
}

impl Options {
    pub fn value(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn has(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }
}

/// Collect `#[name(...)]` options, rejecting any key or flag not in
/// `allowed` (a typo should be a compile error, not silently ignored).
pub fn parse(attrs: &[Attribute], name: &str, allowed: &[&str]) -> syn::Result<Options> {
    let mut options = Options::default();
    for attr in attrs.iter().filter(|a| a.path().is_ident(name)) {
        attr.parse_nested_meta(|meta| {
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            if !allowed.contains(&key.as_str()) {
                return Err(meta.error(format!(
                    "unknown `{name}` option `{key}` (expected one of: {})",
                    allowed.join(", ")
                )));
            }
            if meta.input.peek(syn::Token![=]) {
                let value: LitStr = meta.value()?.parse()?;
                options.values.push((key, value.value()));
            } else {
                options.flags.push(key);
            }
            Ok(())
        })?;
    }
    Ok(options)
}

/// `UserAccount` -> `user_account`.
pub fn snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
