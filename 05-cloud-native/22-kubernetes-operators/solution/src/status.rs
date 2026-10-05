//! Exercise 4: reporting status.
//!
//! The spec is what the user asked for; the status is what the operator
//! observed. It's how `kubectl get apps` shows progress and how
//! `kubectl wait --for=condition=Ready app/web` knows when to return.

use crate::crd::{App, AppStatus, Condition, Phase};
use k8s_openapi::api::apps::v1::Deployment;

fn condition(ready: bool, reason: &str, message: String) -> Condition {
    Condition {
        type_: "Ready".into(),
        status: if ready { "True" } else { "False" }.into(),
        reason: reason.into(),
        message,
    }
}

/// Status of an App given its Deployment (if it exists yet).
pub fn app_status(app: &App, deployment: Option<&Deployment>) -> AppStatus {
    let generation = app.metadata.generation;
    let desired = app.spec.replicas;
    let Some(deployment) = deployment else {
        return AppStatus {
            phase: Phase::Pending,
            ready_replicas: 0,
            observed_generation: generation,
            conditions: vec![condition(
                false,
                "NotCreated",
                "the Deployment doesn't exist yet".into(),
            )],
        };
    };

    let status = deployment.status.clone().unwrap_or_default();
    let ready = status.ready_replicas.unwrap_or(0);
    let updated = status.updated_replicas.unwrap_or(0);
    let seen = status.observed_generation == deployment.metadata.generation;
    let stuck = status.conditions.unwrap_or_default().iter().any(|c| {
        c.type_ == "Progressing"
            && c.status == "False"
            && c.reason.as_deref() == Some("ProgressDeadlineExceeded")
    });
    let counts = format!("{ready}/{desired} replicas ready");

    let (phase, cond) = if stuck {
        (
            Phase::Degraded,
            condition(
                false,
                "ProgressDeadlineExceeded",
                format!("rollout stuck: {counts}"),
            ),
        )
    } else if seen && ready == desired && updated == desired {
        (Phase::Ready, condition(true, "AllReplicasReady", counts))
    } else {
        (Phase::Progressing, condition(false, "RollingOut", counts))
    };
    AppStatus {
        phase,
        ready_replicas: ready,
        observed_generation: generation,
        conditions: vec![cond],
    }
}

/// Status for a spec that failed validation: nothing is created.
pub fn invalid_status(app: &App, problems: &[String]) -> AppStatus {
    AppStatus {
        phase: Phase::Degraded,
        ready_replicas: 0,
        observed_generation: app.metadata.generation,
        conditions: vec![condition(false, "InvalidSpec", problems.join("; "))],
    }
}
