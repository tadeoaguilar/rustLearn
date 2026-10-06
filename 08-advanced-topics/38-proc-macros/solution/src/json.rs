//! The runtime half of Exercise 1: the `ToJson` trait and its impls for
//! standard types (provided). `#[derive(ToJson)]` generates impls for your
//! types that call these.

use std::collections::BTreeMap;

pub trait ToJson {
    fn to_json(&self) -> String;
}

/// A Rust string as a JSON string literal, escaped.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

macro_rules! numbers {
    ($($t:ty),*) => {$(
        impl ToJson for $t {
            fn to_json(&self) -> String {
                self.to_string()
            }
        }
    )*};
}
numbers!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);

impl ToJson for f64 {
    /// JSON has no NaN or infinity: they become `null`.
    fn to_json(&self) -> String {
        if self.is_finite() {
            self.to_string()
        } else {
            "null".into()
        }
    }
}

impl ToJson for f32 {
    fn to_json(&self) -> String {
        (*self as f64).to_json()
    }
}

impl ToJson for bool {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl ToJson for str {
    fn to_json(&self) -> String {
        escape(self)
    }
}

impl ToJson for String {
    fn to_json(&self) -> String {
        escape(self)
    }
}

impl<T: ToJson + ?Sized> ToJson for &T {
    fn to_json(&self) -> String {
        (**self).to_json()
    }
}

impl<T: ToJson + ?Sized> ToJson for Box<T> {
    fn to_json(&self) -> String {
        (**self).to_json()
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> String {
        self.as_ref().map_or_else(|| "null".into(), ToJson::to_json)
    }
}

impl<T: ToJson> ToJson for [T] {
    fn to_json(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(ToJson::to_json)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> String {
        self.as_slice().to_json()
    }
}

impl<V: ToJson> ToJson for BTreeMap<String, V> {
    fn to_json(&self) -> String {
        let parts: Vec<String> = self
            .iter()
            .map(|(k, v)| format!("{}:{}", escape(k), v.to_json()))
            .collect();
        format!("{{{}}}", parts.join(","))
    }
}
