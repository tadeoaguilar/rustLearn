//! Exercise 3: `tasks` -- a task manager with a JSON store.
//!
//! State that outlives the process goes in a file. Two things make that
//! robust: the file location is configurable (`--file`, or the `TASKS_FILE`
//! environment variable), and saving is *atomic* -- write a temporary file
//! next to the real one, then rename it over -- so a crash mid-write never
//! leaves a half-written store behind.

use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub done: bool,
    pub priority: Priority,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// `YYYY-MM-DD`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("no task with id {0}")]
    NotFound(u32),
    #[error("not a date (YYYY-MM-DD): {0:?}")]
    InvalidDate(String),
    #[error("a title can't be empty")]
    EmptyTitle,
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("the store is corrupt: {0}")]
    Json(#[from] serde_json::Error),
}

/// What's on disk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreData {
    pub next_id: u32,
    pub tasks: Vec<Task>,
}

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    data: StoreData,
}

/// Which tasks `list` shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListFilter {
    pub include_done: bool,
    pub tag: Option<String>,
    pub priority: Option<Priority>,
}

/// `YYYY-MM-DD` with a plausible month and day.
pub fn valid_date(s: &str) -> bool {
    todo!("Exercise 3")
}

impl Store {
    /// Open the store at `path`; a missing file is an empty store.
    pub fn load(path: impl Into<PathBuf>) -> Result<Store, TaskError> {
        todo!("Exercise 3")
    }

    /// Write atomically: a temp file in the same directory, then rename.
    pub fn save(&self) -> Result<(), TaskError> {
        todo!("Exercise 3")
    }

    pub fn tasks(&self) -> &[Task] {
        todo!("Exercise 3")
    }

    pub fn add(
        &mut self,
        title: &str,
        priority: Priority,
        tags: Vec<String>,
        due: Option<String>,
    ) -> Result<u32, TaskError> {
        todo!("Exercise 3")
    }

    fn get_mut(&mut self, id: u32) -> Result<&mut Task, TaskError> {
        todo!("Exercise 3")
    }

    pub fn complete(&mut self, id: u32) -> Result<(), TaskError> {
        todo!("Exercise 3")
    }

    /// Ids are never reused, even after removal.
    pub fn remove(&mut self, id: u32) -> Result<Task, TaskError> {
        todo!("Exercise 3")
    }

    pub fn edit(
        &mut self,
        id: u32,
        title: Option<&str>,
        priority: Option<Priority>,
    ) -> Result<(), TaskError> {
        todo!("Exercise 3")
    }

    /// Open tasks first, then by priority (high first), then by id.
    pub fn list(&self, filter: &ListFilter) -> Vec<&Task> {
        todo!("Exercise 3")
    }
}

/// A plain-text table: `  3 [x] high   Write tests  #rust  (due 2026-10-31)`.
pub fn format_table(tasks: &[&Task]) -> String {
    todo!("Exercise 3")
}

#[derive(Parser, Debug)]
#[command(name = "tasks", about = "A small task manager")]
pub struct TasksCli {
    /// The store file.
    // TODO Exercise 3: #[arg(...)]
    pub file: PathBuf,
    #[command(subcommand)]
    pub command: TaskCommand,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum TaskCommand {
    /// Add a task.
    Add {
        title: String,
        // TODO Exercise 3: #[arg(...)]
        priority: Priority,
        /// May be repeated: -t work -t urgent
        // TODO Exercise 3: #[arg(...)]
        tags: Vec<String>,
        // TODO Exercise 3: #[arg(...)]
        due: Option<String>,
    },
    /// List tasks.
    List {
        /// Include finished tasks.
        // TODO Exercise 3: #[arg(...)]
        all: bool,
        // TODO Exercise 3: #[arg(...)]
        tag: Option<String>,
        // TODO Exercise 3: #[arg(...)]
        priority: Option<Priority>,
    },
    /// Mark a task done.
    Done { id: u32 },
    /// Delete a task.
    Remove { id: u32 },
    /// Change a task.
    Edit {
        id: u32,
        // TODO Exercise 3: #[arg(...)]
        title: Option<String>,
        // TODO Exercise 3: #[arg(...)]
        priority: Option<Priority>,
    },
}

/// Run one command against the store in `cli.file`, saving if it changed.
pub fn run(cli: &TasksCli, out: &mut impl Write) -> Result<(), TaskError> {
    todo!("Exercise 3")
}
