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
    todo!("Exercise 1")
}

fn default_priority() -> u8 {
    todo!("Exercise 1")
}

impl TaskInput {
    pub fn new(title: &str) -> Self {
        todo!("Exercise 1")
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskStore {
    inner: Arc<RwLock<(BTreeMap<u64, Task>, u64)>>, // (tasks, last id)
}

impl TaskStore {
    pub fn new() -> Self {
        todo!("Exercise 1")
    }

    /// `n` generated tasks: priorities cycle 1..=5, statuses cycle.
    pub fn with_generated(n: usize) -> Self {
        todo!("Exercise 1")
    }

    /// Input must already be validated -- that's the extractor's job.
    pub fn create(&self, input: TaskInput) -> Task {
        todo!("Exercise 1")
    }

    pub fn get(&self, id: u64) -> Option<Task> {
        todo!("Exercise 1")
    }

    pub fn all(&self) -> Vec<Task> {
        todo!("Exercise 1")
    }

    pub fn replace(&self, id: u64, input: TaskInput) -> Option<Task> {
        todo!("Exercise 1")
    }

    pub fn delete(&self, id: u64) -> bool {
        todo!("Exercise 1")
    }
}
