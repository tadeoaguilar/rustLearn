//! Exercise 7: Compose `depends_on`.
//!
//! `docker compose up` starts services in dependency order, and with
//! `condition: service_healthy` it waits for a dependency's health check to
//! pass first. Computing that order -- and catching the mistakes Compose
//! would only report at `up` time -- is a topological sort.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// Just started (the default): no guarantee it's *ready*.
    #[default]
    ServiceStarted,
    /// Its health check passed.
    ServiceHealthy,
    /// It ran to completion with exit code 0 (migrations, seeding).
    ServiceCompletedSuccessfully,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum DependsOnSyntax {
    /// `depends_on: [db, cache]`
    Short(Vec<String>),
    /// `depends_on: { db: { condition: service_healthy } }`
    Long(BTreeMap<String, LongDependency>),
}

#[derive(Debug, Clone, Deserialize)]
struct LongDependency {
    #[serde(default)]
    condition: Condition,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub image: Option<String>,
    #[serde(default, rename = "depends_on")]
    depends_on_raw: Option<DependsOnSyntax>,
    pub healthcheck: Option<serde_yaml_ng::Value>,
}

impl Service {
    /// Both syntaxes, normalised.
    pub fn depends_on(&self) -> BTreeMap<String, Condition> {
        todo!("Exercise 7")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposeFile {
    pub services: BTreeMap<String, Service>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError {
    Parse(String),
    UnknownDependency {
        service: String,
        dependency: String,
    },
    /// `service_healthy` on a service without a health check: it would
    /// never become healthy.
    NoHealthcheck {
        service: String,
        dependency: String,
    },
    /// The services involved in a cycle, sorted.
    Cycle(Vec<String>),
}

pub fn parse(yaml: &str) -> Result<ComposeFile, ComposeError> {
    todo!("Exercise 7")
}

/// Start order in *waves*: every service in a wave depends only on earlier
/// waves, so a wave's services can start in parallel. Names sorted in a wave.
pub fn startup_order(file: &ComposeFile) -> Result<Vec<Vec<String>>, ComposeError> {
    todo!("Exercise 7")
}
