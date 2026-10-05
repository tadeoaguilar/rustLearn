//! Exercise 1: the domain model and an in-memory store.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

/// A task as stored. `version` increments on every change (Bonus: ETags).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    /// 1 (lowest) to 5 (highest)
    pub priority: u8,
    pub version: u32,
}

/// The request body for POST and PUT. `validator` checks the rules declared
/// in the attributes; `#[serde(default)]` makes fields optional in JSON.
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct TaskInput {
    #[validate(length(min = 1, max = 100, message = "must be 1 to 100 characters"))]
    pub title: String,
    #[validate(length(max = 1000, message = "must be at most 1000 characters"))]
    pub description: Option<String>,
    #[serde(default = "default_status")]
    pub status: Status,
    #[validate(range(min = 1, max = 5, message = "must be between 1 and 5"))]
    #[serde(default = "default_priority")]
    pub priority: u8,
}

fn default_status() -> Status {
    Status::Todo
}

fn default_priority() -> u8 {
    3
}

impl TaskInput {
    pub fn new(title: &str) -> Self {
        TaskInput {
            title: title.into(),
            description: None,
            status: Status::Todo,
            priority: 3,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskStore {
    inner: Arc<RwLock<(BTreeMap<u64, Task>, u64)>>, // (tasks, last id)
}

impl TaskStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// `n` generated tasks: priorities cycle 1..=5, statuses cycle.
    pub fn with_generated(n: usize) -> Self {
        let store = Self::new();
        for i in 1..=n {
            let status = [Status::Todo, Status::InProgress, Status::Done][i % 3];
            store.create(TaskInput {
                title: format!("Task {i}"),
                description: None,
                status,
                priority: (i % 5) as u8 + 1,
            });
        }
        store
    }

    /// Input must already be validated -- that's the extractor's job.
    pub fn create(&self, input: TaskInput) -> Task {
        let mut guard = self.inner.write().unwrap();
        guard.1 += 1;
        let id = guard.1;
        let task = Task {
            id,
            title: input.title.trim().to_string(),
            description: input.description,
            status: input.status,
            priority: input.priority,
            version: 1,
        };
        guard.0.insert(id, task.clone());
        task
    }

    pub fn get(&self, id: u64) -> Option<Task> {
        self.inner.read().unwrap().0.get(&id).cloned()
    }

    pub fn all(&self) -> Vec<Task> {
        self.inner.read().unwrap().0.values().cloned().collect()
    }

    pub fn replace(&self, id: u64, input: TaskInput) -> Option<Task> {
        let mut guard = self.inner.write().unwrap();
        let task = guard.0.get_mut(&id)?;
        task.title = input.title.trim().to_string();
        task.description = input.description;
        task.status = input.status;
        task.priority = input.priority;
        task.version += 1;
        Some(task.clone())
    }

    pub fn delete(&self, id: u64) -> bool {
        self.inner.write().unwrap().0.remove(&id).is_some()
    }
}
