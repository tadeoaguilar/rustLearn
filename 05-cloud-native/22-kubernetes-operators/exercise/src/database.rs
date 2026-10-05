//! Exercise 5: a database operator -- external resources and finalizers.
//!
//! An App's children live in the cluster, so owner references clean them up.
//! A `Database` is provisioned *outside* the cluster (a cloud provider, a DBA
//! team's API). Kubernetes can't delete that, so the operator puts a
//! **finalizer** on the Database: the API server then won't remove the object
//! until the operator has cleaned up and taken the finalizer off.
//!
//! ```text
//! create:  add finalizer FIRST -> provision -> credentials Secret -> status
//! delete:  (deletionTimestamp set) -> deprovision -> remove finalizer -> gone
//! ```
//!
//! Adding the finalizer first matters: if the operator crashed after
//! provisioning but before adding it, a delete would skip the cleanup and
//! leak the database.

use crate::crd::{Database, DatabaseSpec, Engine};
use k8s_openapi::ByteString;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{Resource, ResourceExt};
use rand::Rng;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

pub const FINALIZER: &str = "rustlearn.dev/cleanup";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    /// Temporary: retry later.
    #[error("database provider unavailable")]
    Unavailable,
    /// Permanent until the spec changes: retrying won't help.
    #[error("{0}")]
    Invalid(String),
}

/// The outside system. Both calls must be idempotent: a reconcile can run
/// any number of times, and crash at any point.
pub trait DbProvider: Send + Sync {
    fn ensure(&self, id: &str, spec: &DatabaseSpec) -> Result<Endpoint, ProviderError>;
    fn delete(&self, id: &str) -> Result<(), ProviderError>;
}

/// PROVIDED: an in-memory provider for tests and the demo.
#[derive(Default)]
pub struct FakeDbProvider {
    databases: Mutex<BTreeMap<String, DatabaseSpec>>,
    fail_next: AtomicU32,
}

impl FakeDbProvider {
    pub fn new() -> FakeDbProvider {
        FakeDbProvider::default()
    }

    pub fn databases(&self) -> Vec<String> {
        self.databases.lock().unwrap().keys().cloned().collect()
    }

    pub fn fail_next(&self, n: u32) {
        self.fail_next.store(n, Ordering::SeqCst);
    }

    fn failing(&self) -> bool {
        self.fail_next
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .is_ok()
    }
}

impl DbProvider for FakeDbProvider {
    fn ensure(&self, id: &str, spec: &DatabaseSpec) -> Result<Endpoint, ProviderError> {
        if self.failing() {
            return Err(ProviderError::Unavailable);
        }
        let mut dbs = self.databases.lock().unwrap();
        if let Some(existing) = dbs.get(id) {
            if existing.engine != spec.engine {
                return Err(ProviderError::Invalid(
                    "the engine of an existing database can't change".into(),
                ));
            }
            if spec.storage_gb < existing.storage_gb {
                return Err(ProviderError::Invalid(format!(
                    "storage can't shrink ({} GB -> {} GB)",
                    existing.storage_gb, spec.storage_gb
                )));
            }
        }
        dbs.insert(id.to_string(), spec.clone());
        let port = match spec.engine {
            Engine::Postgres => 5432,
            Engine::Mysql => 3306,
        };
        Ok(Endpoint {
            host: format!("{}.db.internal", id.replace('/', "-")),
            port,
        })
    }

    fn delete(&self, id: &str) -> Result<(), ProviderError> {
        if self.failing() {
            return Err(ProviderError::Unavailable);
        }
        self.databases.lock().unwrap().remove(id);
        Ok(())
    }
}

pub fn has_finalizer(obj: &impl ResourceExt) -> bool {
    todo!("Exercise 5")
}

/// The finalizer list with ours added (no duplicates).
pub fn with_finalizer(finalizers: &[String]) -> Vec<String> {
    todo!("Exercise 5")
}

/// The finalizer list with ours removed -- other controllers' stay.
pub fn without_finalizer(finalizers: &[String]) -> Vec<String> {
    todo!("Exercise 5")
}

pub fn secret_name(db: &Database) -> String {
    todo!("Exercise 5")
}

pub fn generate_password(len: usize) -> String {
    todo!("Exercise 5")
}

/// The password already stored in the Secret, if any. Reconciles must reuse
/// it: generating a new one on every reconcile would lock every client out.
pub fn password_from(secret: &Secret) -> Option<String> {
    todo!("Exercise 5")
}

/// The Secret apps use to connect. Owned by the Database, so it's deleted
/// with it.
pub fn credentials_secret(db: &Database, endpoint: &Endpoint, password: &str) -> Secret {
    todo!("Exercise 5")
}
