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

// Several `printcolumn(name = .., type_ = ..)` repeat the same keys by design;
// clippy mistakes that for a duplicated attribute.
#[allow(clippy::duplicated_attributes)]
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[kube(
    group = "rustlearn.dev",
    version = "v1",
    kind = "App",
    namespaced,
    status = "AppStatus",
    shortname = "rapp",
    derive = "PartialEq",
    printcolumn(name = "Image", type_ = "string", json_path = ".spec.image"),
    printcolumn(name = "Desired", type_ = "integer", json_path = ".spec.replicas"),
    printcolumn(name = "Ready", type_ = "integer", json_path = ".status.readyReplicas"),
    printcolumn(name = "Phase", type_ = "string", json_path = ".status.phase")
)]
#[serde(rename_all = "camelCase")]
pub struct AppSpec {
    /// Container image, with a tag or digest.
    #[schemars(length(min = 1))]
    pub image: String,
    #[schemars(range(min = 0, max = 50))]
    pub replicas: i32,
    #[serde(default = "default_port")]
    #[schemars(range(min = 1, max = 65535))]
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

// Several `printcolumn(name = .., type_ = ..)` repeat the same keys by design;
// clippy mistakes that for a duplicated attribute.
#[allow(clippy::duplicated_attributes)]
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[kube(
    group = "rustlearn.dev",
    version = "v1",
    kind = "Database",
    namespaced,
    status = "DatabaseStatus",
    shortname = "rdb",
    derive = "PartialEq",
    printcolumn(name = "Engine", type_ = "string", json_path = ".spec.engine"),
    printcolumn(name = "Phase", type_ = "string", json_path = ".status.phase"),
    printcolumn(name = "Endpoint", type_ = "string", json_path = ".status.endpoint")
)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSpec {
    pub engine: Engine,
    #[schemars(length(min = 1))]
    pub version: String,
    #[schemars(range(min = 1, max = 1000))]
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
    use kube::CustomResourceExt;
    let docs = [App::crd(), Database::crd()]
        .map(|crd| serde_yaml_ng::to_string(&crd).expect("CRDs serialise"));
    docs.join("---\n")
}

/// Checks the schema can't express. (CEL rules, `x-kubernetes-validations`,
/// can do some of this in the API server; an operator checks anyway, since
/// objects may predate a rule.)
pub fn validate_app(spec: &AppSpec) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    let last = spec.image.rsplit('/').next().unwrap_or("");
    if !spec.image.contains('@') && !last.contains(':') {
        problems.push(format!("image {:?} has no tag: pin a version", spec.image));
    } else if last.ends_with(":latest") {
        problems.push(format!(
            "image {:?} uses :latest: pin a version",
            spec.image
        ));
    }
    for name in spec.env.keys() {
        let valid = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid {
            problems.push(format!("env var name {name:?} is not a valid identifier"));
        }
    }
    for key in spec.config.keys() {
        let valid = !key.is_empty()
            && key.len() <= 253
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-._".contains(c));
        if !valid {
            problems.push(format!("config key {key:?} is not a valid file name"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}
