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
        match self {
            NoteError::NotFound(id) => write!(f, "note {id} not found"),
            NoteError::Invalid(why) => write!(f, "invalid note: {why}"),
        }
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
        let title = self.title.trim().to_string();
        if title.is_empty() {
            return Err(NoteError::Invalid("title must not be empty".into()));
        }
        if title.chars().count() > 120 {
            return Err(NoteError::Invalid(
                "title must be at most 120 characters".into(),
            ));
        }
        let mut tags: Vec<String> = self
            .tags
            .iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        tags.sort();
        tags.dedup();
        Ok(NoteInput {
            title,
            body: self.body,
            tags,
        })
    }
}

impl NoteStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// A store with three notes, for demos.
    pub fn with_samples() -> Self {
        let store = Self::new();
        for (title, body, tags) in [
            (
                "Axum",
                "Tower-based, extractors everywhere",
                vec!["rust", "web"],
            ),
            (
                "Actix-web",
                "Fast, mature, its own runtime model",
                vec!["rust", "web"],
            ),
            ("Groceries", "milk, eggs", vec!["home"]),
        ] {
            let input = NoteInput {
                title: title.into(),
                body: body.into(),
                tags: tags.into_iter().map(String::from).collect(),
            };
            store.create(input).expect("sample notes are valid");
        }
        store
    }

    pub fn create(&self, input: NoteInput) -> Result<Note, NoteError> {
        let input = input.validated()?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let note = Note {
            id,
            title: input.title,
            body: input.body,
            tags: input.tags,
        };
        self.notes.write().unwrap().insert(id, note.clone());
        Ok(note)
    }

    pub fn get(&self, id: u64) -> Result<Note, NoteError> {
        self.notes
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(NoteError::NotFound(id))
    }

    /// In id order. `tag` must match exactly; `q` is a case-insensitive
    /// substring of title or body.
    pub fn list(&self, filter: &NoteFilter) -> Vec<Note> {
        let q = filter.q.as_ref().map(|q| q.to_lowercase());
        let tag = filter.tag.as_ref().map(|t| t.to_lowercase());
        self.notes
            .read()
            .unwrap()
            .values()
            .filter(|n| tag.as_ref().is_none_or(|t| n.tags.contains(t)))
            .filter(|n| {
                q.as_ref().is_none_or(|q| {
                    n.title.to_lowercase().contains(q) || n.body.to_lowercase().contains(q)
                })
            })
            .cloned()
            .collect()
    }

    /// Replaces the whole note (PUT semantics).
    pub fn update(&self, id: u64, input: NoteInput) -> Result<Note, NoteError> {
        let input = input.validated()?;
        let mut notes = self.notes.write().unwrap();
        let note = notes.get_mut(&id).ok_or(NoteError::NotFound(id))?;
        *note = Note {
            id,
            title: input.title,
            body: input.body,
            tags: input.tags,
        };
        Ok(note.clone())
    }

    pub fn delete(&self, id: u64) -> Result<(), NoteError> {
        self.notes
            .write()
            .unwrap()
            .remove(&id)
            .map(|_| ())
            .ok_or(NoteError::NotFound(id))
    }

    pub fn len(&self) -> usize {
        self.notes.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
