//! Module 22 -- Kubernetes Operators. YOUR WORKSPACE.
//!
//! An operator is a program that manages an application the way a human
//! operator would: it watches custom resources (`kind: App`, `kind:
//! Database`) and keeps making the cluster match them.
//!
//! | File            | Exercise |
//! |-----------------|----------|
//! | `crd.rs`        | 1  the custom resources and their schemas |
//! | `resources.rs`  | 2  desired state: ConfigMap, Deployment, Service |
//! | `plan.rs`       | 3  diffing desired against observed (without hot loops) |
//! | `status.rs`     | 4  status and conditions |
//! | `database.rs`   | 5  an external resource with a finalizer |
//! | `reconcile.rs`  | 6  the reconcilers, and a converge-loop test harness |
//! | `metrics.rs`    | 7  per-object backoff and Prometheus metrics |
//! | `cluster.rs`    | the `Cluster` trait and `FakeCluster` (provided) |
//! | `controller.rs` | the real-cluster wiring with kube-rs (provided) |

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod cluster;
pub mod controller;
pub mod crd;
pub mod database;
pub mod metrics;
pub mod plan;
pub mod reconcile;
pub mod resources;
pub mod status;

/// The `deploy/` directory: generated CRDs, RBAC and example resources.
pub const DEPLOY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../deploy");
