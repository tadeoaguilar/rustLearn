//! Exercise 1: custom resources.
//!
//! A CustomResourceDefinition teaches the Kubernetes API a new kind of
//! object. kube-rs derives the CRD -- including an OpenAPI schema the API
//! server validates against -- from a Rust struct, so the Rust types and the
//! cluster can't disagree.
//!
//! Two kinds:
//!
//! ```yaml
//! apiVersion: rustlearn.dev/v1
//! kind: App               # rolled out as a ConfigMap + Deployment + Service
//! spec: { image: ghcr.io/acme/web:1.4.2, replicas: 3, port: 8080, env: {...}, config: {...} }
//! ---
//! kind: Database          # an *external* database, plus a Secret with credentials
//! spec: { engine: postgres, version: "17", storageGb: 20 }
//! ```

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const GROUP: &str = "rustlearn.dev";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[kube(
    group = "rustlearn.dev",
    version = "v1",
    kind = "App",
    namespaced,
    status = "AppStatus",
    shortname = "rapp",
    derive = "PartialEq"
)]
#[serde(rename_all = "camelCase")]
// Exercise 1: add schema validation the API server will enforce --
// replicas 0..=50, port 1..=65535, a non-empty image (`#[schemars(range(..))]`,
// `#[schemars(length(..))]`) -- and printer columns for `kubectl get apps`.
pub struct AppSpec {
    /// Container image, with a tag or digest.
    pub image: String,
    pub replicas: i32,
    #[serde(default = "default_port")]
    pub port: i32,
    /// Environment variables for the container.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Files for the app, mounted at /etc/app from a ConfigMap.
    #[serde(default)]
    pub config: BTreeMap<String, String>,
}

fn default_port() -> i32 {
    8080
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Default)]
pub enum Phase {
    #[default]
    Pending,
    Progressing,
    Ready,
    Degraded,
    Deleting,
}

/// The standard shape of a status condition (`kubectl wait --for=condition=Ready`).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    #[serde(rename = "type")]
    pub type_: String,
    /// "True", "False" or "Unknown" -- strings, by Kubernetes convention.
    pub status: String,
    pub reason: String,
    pub message: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub phase: Phase,
    pub ready_replicas: i32,
    /// The `metadata.generation` this status describes. Clients compare it
    /// with the current generation to know whether the status is stale.
    pub observed_generation: Option<i64>,
    pub conditions: Vec<Condition>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Postgres,
    Mysql,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[kube(
    group = "rustlearn.dev",
    version = "v1",
    kind = "Database",
    namespaced,
    status = "DatabaseStatus",
    shortname = "rdb",
    derive = "PartialEq"
)]
#[serde(rename_all = "camelCase")]
// Exercise 1: storageGb 1..=1000, a non-empty version; printer columns.
pub struct DatabaseSpec {
    pub engine: Engine,
    pub version: String,
    pub storage_gb: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub phase: Phase,
    pub endpoint: Option<String>,
    /// The Secret holding username, password, host, port and url.
    pub secret_name: Option<String>,
    pub message: Option<String>,
}

/// Both CRDs as one multi-document YAML file -- what `kubectl apply -f` takes.
pub fn crds_yaml() -> String {
    todo!("Exercise 1")
}

/// Checks the schema can't express. (CEL rules, `x-kubernetes-validations`,
/// can do some of this in the API server; an operator checks anyway, since
/// objects may predate a rule.)
pub fn validate_app(spec: &AppSpec) -> Result<(), Vec<String>> {
    todo!("Exercise 1")
}
