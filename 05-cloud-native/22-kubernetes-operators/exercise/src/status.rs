//! Exercise 4: reporting status.
//!
//! The spec is what the user asked for; the status is what the operator
//! observed. It's how `kubectl get apps` shows progress and how
//! `kubectl wait --for=condition=Ready app/web` knows when to return.

use crate::crd::{App, AppStatus, Condition, Phase};
use k8s_openapi::api::apps::v1::Deployment;

fn condition(ready: bool, reason: &str, message: String) -> Condition {
    todo!("Exercise 4")
}

/// Status of an App given its Deployment (if it exists yet).
pub fn app_status(app: &App, deployment: Option<&Deployment>) -> AppStatus {
    todo!("Exercise 4")
}

/// Status for a spec that failed validation: nothing is created.
pub fn invalid_status(app: &App, problems: &[String]) -> AppStatus {
    todo!("Exercise 4")
}
