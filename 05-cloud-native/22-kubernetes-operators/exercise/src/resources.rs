//! Exercise 2: the desired state.
//!
//! An operator's core is a pure function: custom resource in, the objects
//! that should exist out. Everything it creates carries:
//!
//! - the standard `app.kubernetes.io/*` labels, so `kubectl get -l` and
//!   dashboards find it;
//! - an **owner reference** to the App, so deleting the App makes the
//!   cluster's garbage collector delete them too;
//! - (the Deployment's pod template) a **hash of the config**: a ConfigMap
//!   change doesn't restart pods by itself, but a changed pod-template
//!   annotation does -- a rolling update.

use crate::crd::App;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    ConfigMap, ConfigMapVolumeSource, Container, ContainerPort, EnvVar, HTTPGetAction,
    PodSecurityContext, PodSpec, PodTemplateSpec, Probe, SecurityContext, Service, ServicePort,
    ServiceSpec, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{Resource, ResourceExt};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const MANAGER: &str = "rustlearn-operator";
pub const CONFIG_HASH: &str = "rustlearn.dev/config-hash";
pub const CONFIG_MOUNT: &str = "/etc/app";

/// The labels every object of this App gets. The selector uses only the
/// first two: selectors are immutable, so they must never include anything
/// that changes (like a version).
pub fn labels(app: &App) -> BTreeMap<String, String> {
    todo!("Exercise 2")
}

pub fn selector(app: &App) -> BTreeMap<String, String> {
    todo!("Exercise 2")
}

/// Name, namespace, labels and the owner reference shared by all children.
fn child_meta(app: &App, name: String) -> ObjectMeta {
    todo!("Exercise 2")
}

/// First 16 hex digits of SHA-256 over the sorted config entries.
pub fn config_hash(config: &BTreeMap<String, String>) -> String {
    todo!("Exercise 2")
}

pub fn desired_configmap(app: &App) -> ConfigMap {
    todo!("Exercise 2")
}

fn http_probe(path: &str, port: i32, period: i32) -> Probe {
    todo!("Exercise 2")
}

pub fn desired_deployment(app: &App) -> Deployment {
    todo!("Exercise 2")
}

pub fn desired_service(app: &App) -> Service {
    todo!("Exercise 2")
}

/// Everything an App should own.
#[derive(Debug, Clone, PartialEq)]
pub struct Desired {
    pub configmap: ConfigMap,
    pub deployment: Deployment,
    pub service: Service,
}

pub fn desired(app: &App) -> Desired {
    todo!("Exercise 2")
}
