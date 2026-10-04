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
        todo!("Bonus")
    }

    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        todo!("Bonus")
    }

    pub fn is_null(&self) -> bool {
        todo!("Bonus")
    }

    pub fn as_bool(&self) -> Option<bool> {
        todo!("Bonus")
    }

    pub fn as_number(&self) -> Option<f64> {
        todo!("Bonus")
    }

    pub fn as_str(&self) -> Option<&str> {
        todo!("Bonus")
    }

    /// Pretty JSON, `indent` spaces per level. Object keys are sorted because
    /// HashMap iteration order is random -- unsorted output would differ from
    /// run to run and be untestable.
    pub fn to_pretty_string(&self, indent: usize) -> String {
        todo!("Bonus")
    }

    fn write_pretty(&self, out: &mut String, indent: usize, depth: usize) {
        todo!("Bonus")
    }
}

/// Compact, single-line JSON (also used for scalars by the pretty printer).
impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Bonus")
    }
}

/// Quote a string and escape the characters JSON requires.
fn escape(s: &str) -> String {
    todo!("Bonus")
}

// `From` impls make building values much less noisy: `"Alice".into()`.
impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        todo!("Bonus")
    }
}
impl From<f64> for JsonValue {
    fn from(n: f64) -> Self {
        todo!("Bonus")
    }
}
impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        todo!("Bonus")
    }
}
impl From<Vec<JsonValue>> for JsonValue {
    fn from(v: Vec<JsonValue>) -> Self {
        todo!("Bonus")
    }
}

pub fn sample_user() -> JsonValue {
    todo!("Bonus")
}

pub fn run() {
    todo!("Bonus")
}
