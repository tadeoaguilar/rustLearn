//! Exercise 1: the framework-agnostic core.
//!
//! Nothing here knows about HTTP. That's the point: the same `NoteStore` is
//! used by the Axum app, the Actix app and the tests, and switching
//! frameworks wouldn't touch it.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

/// What clients send to create or replace a note (no id: the server assigns it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteInput {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Query parameters for listing: `?tag=rust&q=async`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct NoteFilter {
    pub tag: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteError {
    NotFound(u64),
    Invalid(String),
}

impl fmt::Display for NoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 1")
    }
}

impl std::error::Error for NoteError {}

/// Cheap to clone: every clone shares the same data (Arc). That's what lets
/// each framework hand a copy to every request handler.
#[derive(Debug, Clone, Default)]
pub struct NoteStore {
    notes: Arc<RwLock<BTreeMap<u64, Note>>>,
    next_id: Arc<AtomicU64>,
}

impl NoteInput {
    /// Trims, normalises tags (lower-case, deduplicated, sorted), validates.
    fn validated(self) -> Result<NoteInput, NoteError> {
        todo!("Exercise 1")
    }
}

impl NoteStore {
    pub fn new() -> Self {
        todo!("Exercise 1")
    }

    /// A store with three notes, for demos.
    pub fn with_samples() -> Self {
        todo!("Exercise 1")
    }

    pub fn create(&self, input: NoteInput) -> Result<Note, NoteError> {
        todo!("Exercise 1")
    }

    pub fn get(&self, id: u64) -> Result<Note, NoteError> {
        todo!("Exercise 1")
    }

    /// In id order. `tag` must match exactly; `q` is a case-insensitive
    /// substring of title or body.
    pub fn list(&self, filter: &NoteFilter) -> Vec<Note> {
        todo!("Exercise 1")
    }

    /// Replaces the whole note (PUT semantics).
    pub fn update(&self, id: u64, input: NoteInput) -> Result<Note, NoteError> {
        todo!("Exercise 1")
    }

    pub fn delete(&self, id: u64) -> Result<(), NoteError> {
        todo!("Exercise 1")
    }

    pub fn len(&self) -> usize {
        todo!("Exercise 1")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 1")
    }
}
