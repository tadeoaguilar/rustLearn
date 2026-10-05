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
        match &self.depends_on_raw {
            None => BTreeMap::new(),
            Some(DependsOnSyntax::Short(names)) => names
                .iter()
                .map(|n| (n.clone(), Condition::ServiceStarted))
                .collect(),
            Some(DependsOnSyntax::Long(map)) => {
                map.iter().map(|(n, d)| (n.clone(), d.condition)).collect()
            }
        }
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
    serde_yaml_ng::from_str(yaml).map_err(|e| ComposeError::Parse(e.to_string()))
}

/// Start order in *waves*: every service in a wave depends only on earlier
/// waves, so a wave's services can start in parallel. Names sorted in a wave.
pub fn startup_order(file: &ComposeFile) -> Result<Vec<Vec<String>>, ComposeError> {
    // Validate first.
    for (name, service) in &file.services {
        for (dep, condition) in service.depends_on() {
            let Some(target) = file.services.get(&dep) else {
                return Err(ComposeError::UnknownDependency {
                    service: name.clone(),
                    dependency: dep,
                });
            };
            if condition == Condition::ServiceHealthy && target.healthcheck.is_none() {
                return Err(ComposeError::NoHealthcheck {
                    service: name.clone(),
                    dependency: dep,
                });
            }
        }
    }

    // Kahn's algorithm, one wave at a time.
    let mut remaining: BTreeMap<&str, BTreeSet<String>> = file
        .services
        .iter()
        .map(|(n, s)| (n.as_str(), s.depends_on().into_keys().collect()))
        .collect();
    let mut waves = Vec::new();
    while !remaining.is_empty() {
        let wave: Vec<String> = remaining
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(n, _)| n.to_string())
            .collect();
        if wave.is_empty() {
            // Everything left waits on something left: a cycle (plus
            // anything waiting on it).
            return Err(ComposeError::Cycle(
                remaining.keys().map(|s| s.to_string()).collect(),
            ));
        }
        for name in &wave {
            remaining.remove(name.as_str());
        }
        for deps in remaining.values_mut() {
            for name in &wave {
                deps.remove(name);
            }
        }
        waves.push(wave);
    }
    Ok(waves)
}
