//! Exercise 3: what needs to change?
//!
//! Comparing desired and observed objects with `==` doesn't work: the API
//! server *adds* things -- defaults (`protocol: TCP`, probe timeouts,
//! `revisionHistoryLimit: 10`...), a cluster IP, uids, status. A deep
//! comparison always differs, the operator updates on every reconcile, every
//! update triggers another reconcile: a hot loop.
//!
//! So compare only the fields this operator *owns*, normalised the same way
//! on both sides.

use crate::resources::{CONFIG_HASH, Desired};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Container, Probe, Service};
use std::collections::BTreeMap;

/// What currently exists in the cluster.
#[derive(Debug, Clone, Default)]
pub struct Observed {
    pub configmap: Option<ConfigMap>,
    pub deployment: Option<Deployment>,
    pub service: Option<Service>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Create,
    /// The owned fields that differ, e.g. ["replicas", "image"].
    Update(Vec<&'static str>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    pub configmap: Option<Change>,
    pub deployment: Option<Change>,
    pub service: Option<Change>,
}

impl Plan {
    pub fn is_empty(&self) -> bool {
        todo!("Exercise 3")
    }
}

fn change<T>(observed: Option<&T>, diff: impl FnOnce(&T) -> Vec<&'static str>) -> Option<Change> {
    todo!("Exercise 3")
}

pub fn plan(desired: &Desired, observed: &Observed) -> Plan {
    todo!("Exercise 3")
}

fn diff_configmap(d: &ConfigMap, o: &ConfigMap) -> Vec<&'static str> {
    todo!("Exercise 3")
}

fn container(d: &Deployment) -> Option<&Container> {
    todo!("Exercise 3")
}

/// (path, port) of an HTTP probe; timeouts and thresholds are server defaults.
fn probe_key(p: &Option<Probe>) -> Option<(Option<String>, String)> {
    todo!("Exercise 3")
}

fn diff_deployment(d: &Deployment, o: &Deployment) -> Vec<&'static str> {
    todo!("Exercise 3")
}

fn diff_service(d: &Service, o: &Service) -> Vec<&'static str> {
    todo!("Exercise 3")
}
