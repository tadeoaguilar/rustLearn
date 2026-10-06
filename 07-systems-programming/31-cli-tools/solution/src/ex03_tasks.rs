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
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| s[r].parse::<u32>().ok();
    match (num(0..4), num(5..7), num(8..10)) {
        (Some(_), Some(m), Some(d)) => (1..=12).contains(&m) && (1..=31).contains(&d),
        _ => false,
    }
}

impl Store {
    /// Open the store at `path`; a missing file is an empty store.
    pub fn load(path: impl Into<PathBuf>) -> Result<Store, TaskError> {
        let path = path.into();
        let data = match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => StoreData {
                next_id: 1,
                tasks: Vec::new(),
            },
            Err(e) => return Err(e.into()),
        };
        Ok(Store { path, data })
    }

    /// Write atomically: a temp file in the same directory, then rename.
    pub fn save(&self) -> Result<(), TaskError> {
        let dir = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        serde_json::to_writer_pretty(&mut tmp, &self.data)?;
        tmp.write_all(b"\n")?;
        tmp.persist(&self.path).map_err(|e| e.error)?;
        Ok(())
    }

    pub fn tasks(&self) -> &[Task] {
        &self.data.tasks
    }

    pub fn add(
        &mut self,
        title: &str,
        priority: Priority,
        tags: Vec<String>,
        due: Option<String>,
    ) -> Result<u32, TaskError> {
        if title.trim().is_empty() {
            return Err(TaskError::EmptyTitle);
        }
        if let Some(d) = &due {
            if !valid_date(d) {
                return Err(TaskError::InvalidDate(d.clone()));
            }
        }
        let id = self.data.next_id.max(1);
        self.data.next_id = id + 1;
        self.data.tasks.push(Task {
            id,
            title: title.trim().to_string(),
            done: false,
            priority,
            tags,
            due,
        });
        Ok(id)
    }

    fn get_mut(&mut self, id: u32) -> Result<&mut Task, TaskError> {
        self.data
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(TaskError::NotFound(id))
    }

    pub fn complete(&mut self, id: u32) -> Result<(), TaskError> {
        self.get_mut(id)?.done = true;
        Ok(())
    }

    /// Ids are never reused, even after removal.
    pub fn remove(&mut self, id: u32) -> Result<Task, TaskError> {
        let index = self
            .data
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or(TaskError::NotFound(id))?;
        Ok(self.data.tasks.remove(index))
    }

    pub fn edit(
        &mut self,
        id: u32,
        title: Option<&str>,
        priority: Option<Priority>,
    ) -> Result<(), TaskError> {
        if title.is_some_and(|t| t.trim().is_empty()) {
            return Err(TaskError::EmptyTitle);
        }
        let task = self.get_mut(id)?;
        if let Some(t) = title {
            task.title = t.trim().to_string();
        }
        if let Some(p) = priority {
            task.priority = p;
        }
        Ok(())
    }

    /// Open tasks first, then by priority (high first), then by id.
    pub fn list(&self, filter: &ListFilter) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self
            .data
            .tasks
            .iter()
            .filter(|t| filter.include_done || !t.done)
            .filter(|t| filter.tag.as_ref().is_none_or(|tag| t.tags.contains(tag)))
            .filter(|t| filter.priority.is_none_or(|p| t.priority == p))
            .collect();
        tasks.sort_by(|a, b| {
            a.done
                .cmp(&b.done)
                .then(b.priority.cmp(&a.priority))
                .then(a.id.cmp(&b.id))
        });
        tasks
    }
}

/// A plain-text table: `  3 [x] high   Write tests  #rust  (due 2026-10-31)`.
pub fn format_table(tasks: &[&Task]) -> String {
    let mut out = String::new();
    for t in tasks {
        let check = if t.done { "x" } else { " " };
        let priority = format!("{:?}", t.priority).to_lowercase();
        let _ = write!(out, "{:>3} [{check}] {priority:<6} {}", t.id, t.title);
        for tag in &t.tags {
            let _ = write!(out, "  #{tag}");
        }
        if let Some(due) = &t.due {
            let _ = write!(out, "  (due {due})");
        }
        out.push('\n');
    }
    out
}

#[derive(Parser, Debug)]
#[command(name = "tasks", about = "A small task manager")]
pub struct TasksCli {
    /// The store file.
    #[arg(long, env = "TASKS_FILE", default_value = "tasks.json")]
    pub file: PathBuf,
    #[command(subcommand)]
    pub command: TaskCommand,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum TaskCommand {
    /// Add a task.
    Add {
        title: String,
        #[arg(short, long, value_enum, default_value_t = Priority::Medium)]
        priority: Priority,
        /// May be repeated: -t work -t urgent
        #[arg(short, long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        due: Option<String>,
    },
    /// List tasks.
    List {
        /// Include finished tasks.
        #[arg(short, long)]
        all: bool,
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,
    },
    /// Mark a task done.
    Done { id: u32 },
    /// Delete a task.
    Remove { id: u32 },
    /// Change a task.
    Edit {
        id: u32,
        #[arg(long)]
        title: Option<String>,
        #[arg(short, long, value_enum)]
        priority: Option<Priority>,
    },
}

/// Run one command against the store in `cli.file`, saving if it changed.
pub fn run(cli: &TasksCli, out: &mut impl Write) -> Result<(), TaskError> {
    let mut store = Store::load(&cli.file)?;
    match &cli.command {
        TaskCommand::Add {
            title,
            priority,
            tags,
            due,
        } => {
            let id = store.add(title, *priority, tags.clone(), due.clone())?;
            store.save()?;
            writeln!(out, "added task {id}")?;
        }
        TaskCommand::List { all, tag, priority } => {
            let filter = ListFilter {
                include_done: *all,
                tag: tag.clone(),
                priority: *priority,
            };
            write!(out, "{}", format_table(&store.list(&filter)))?;
        }
        TaskCommand::Done { id } => {
            store.complete(*id)?;
            store.save()?;
            writeln!(out, "completed task {id}")?;
        }
        TaskCommand::Remove { id } => {
            let task = store.remove(*id)?;
            store.save()?;
            writeln!(out, "removed task {id}: {}", task.title)?;
        }
        TaskCommand::Edit {
            id,
            title,
            priority,
        } => {
            store.edit(*id, title.as_deref(), *priority)?;
            store.save()?;
            writeln!(out, "updated task {id}")?;
        }
    }
    Ok(())
}
