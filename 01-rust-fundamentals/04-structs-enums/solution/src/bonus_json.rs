//! Bonus Challenge: JSON-like Data Structure.
//!
//! A recursive enum: `Array` and `Object` contain more `JsonValue`s. That
//! works because `Vec` and `HashMap` store their elements on the heap; a
//! variant holding a `JsonValue` *directly* would have infinite size and need
//! a `Box` (see module 10).

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Pretty JSON, `indent` spaces per level. Object keys are sorted because
    /// HashMap iteration order is random -- unsorted output would differ from
    /// run to run and be untestable.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        let mut out = String::new();
        self.write_pretty(&mut out, indent, 0);
        out
    }

    fn write_pretty(&self, out: &mut String, indent: usize, depth: usize) {
        let pad = |d: usize| " ".repeat(indent * d);
        match self {
            JsonValue::Array(items) if items.is_empty() => out.push_str("[]"),
            JsonValue::Object(map) if map.is_empty() => out.push_str("{}"),
            JsonValue::Array(items) => {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    out.push_str(&pad(depth + 1));
                    item.write_pretty(out, indent, depth + 1);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&pad(depth));
                out.push(']');
            }
            JsonValue::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                out.push_str("{\n");
                for (i, key) in keys.iter().enumerate() {
                    out.push_str(&pad(depth + 1));
                    out.push_str(&escape(key));
                    out.push_str(": ");
                    map[*key].write_pretty(out, indent, depth + 1);
                    if i + 1 < keys.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&pad(depth));
                out.push('}');
            }
            scalar => out.push_str(&scalar.to_string()),
        }
    }
}

/// Compact, single-line JSON (also used for scalars by the pretty printer).
impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{b}"),
            JsonValue::Number(n) => write!(f, "{n}"), // 30.0 prints as 30
            JsonValue::String(s) => write!(f, "{}", escape(s)),
            JsonValue::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            JsonValue::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                write!(f, "{{")?;
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}:{}", escape(key), map[*key])?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Quote a string and escape the characters JSON requires.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// `From` impls make building values much less noisy: `"Alice".into()`.
impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        JsonValue::Bool(b)
    }
}
impl From<f64> for JsonValue {
    fn from(n: f64) -> Self {
        JsonValue::Number(n)
    }
}
impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        JsonValue::String(s.to_string())
    }
}
impl From<Vec<JsonValue>> for JsonValue {
    fn from(v: Vec<JsonValue>) -> Self {
        JsonValue::Array(v)
    }
}

pub fn sample_user() -> JsonValue {
    let mut user = HashMap::new();
    user.insert("name".to_string(), JsonValue::String("Alice".to_string()));
    user.insert("age".to_string(), JsonValue::Number(30.0));
    user.insert("active".to_string(), JsonValue::Bool(true));
    user.insert(
        "tags".to_string(),
        vec!["admin".into(), "dev".into()].into(),
    );
    user.insert("manager".to_string(), JsonValue::Null);
    JsonValue::Object(user)
}

pub fn run() {
    let json = sample_user();
    if let Some(name) = json.get("name") {
        println!("Name: {name:?}");
    }
    assert_eq!(json.get("age").and_then(|v| v.as_number()), Some(30.0));
    println!(
        "tags[1] = {:?}",
        json.get("tags")
            .and_then(|t| t.get_index(1))
            .and_then(JsonValue::as_str)
    );
    println!("compact: {json}");
    println!("pretty:\n{}", json.to_pretty_string(2));
}
