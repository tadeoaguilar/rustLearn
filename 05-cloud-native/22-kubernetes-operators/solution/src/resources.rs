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
    BTreeMap::from([
        ("app.kubernetes.io/name".to_string(), app.name_any()),
        (
            "app.kubernetes.io/managed-by".to_string(),
            MANAGER.to_string(),
        ),
        (
            "app.kubernetes.io/part-of".to_string(),
            "rustlearn".to_string(),
        ),
    ])
}

pub fn selector(app: &App) -> BTreeMap<String, String> {
    labels(app)
        .into_iter()
        .filter(|(k, _)| k != "app.kubernetes.io/part-of")
        .collect()
}

/// Name, namespace, labels and the owner reference shared by all children.
fn child_meta(app: &App, name: String) -> ObjectMeta {
    ObjectMeta {
        name: Some(name),
        namespace: app.namespace(),
        labels: Some(labels(app)),
        // None if the App has no uid yet (it was never stored).
        owner_references: app.controller_owner_ref(&()).map(|r| vec![r]),
        ..ObjectMeta::default()
    }
}

/// First 16 hex digits of SHA-256 over the sorted config entries.
pub fn config_hash(config: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    for (k, v) in config {
        // Length prefixes: {"a": "bc"} and {"ab": "c"} must hash differently.
        hasher.update((k.len() as u64).to_le_bytes());
        hasher.update(k.as_bytes());
        hasher.update((v.len() as u64).to_le_bytes());
        hasher.update(v.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn desired_configmap(app: &App) -> ConfigMap {
    ConfigMap {
        metadata: child_meta(app, format!("{}-config", app.name_any())),
        data: Some(app.spec.config.clone()),
        ..ConfigMap::default()
    }
}

fn http_probe(path: &str, port: i32, period: i32) -> Probe {
    Probe {
        http_get: Some(HTTPGetAction {
            path: Some(path.into()),
            port: IntOrString::Int(port),
            ..HTTPGetAction::default()
        }),
        period_seconds: Some(period),
        ..Probe::default()
    }
}

pub fn desired_deployment(app: &App) -> Deployment {
    let name = app.name_any();
    let spec = &app.spec;
    let env: Vec<EnvVar> = spec
        .env
        .iter()
        .map(|(k, v)| EnvVar {
            name: k.clone(),
            value: Some(v.clone()),
            ..EnvVar::default()
        })
        .collect();
    let container = Container {
        name: "app".into(),
        image: Some(spec.image.clone()),
        ports: Some(vec![ContainerPort {
            container_port: spec.port,
            name: Some("http".into()),
            ..ContainerPort::default()
        }]),
        env: Some(env),
        // The endpoints module 21's service provides.
        liveness_probe: Some(http_probe("/healthz", spec.port, 10)),
        readiness_probe: Some(http_probe("/readyz", spec.port, 5)),
        volume_mounts: Some(vec![VolumeMount {
            name: "config".into(),
            mount_path: CONFIG_MOUNT.into(),
            read_only: Some(true),
            ..VolumeMount::default()
        }]),
        security_context: Some(SecurityContext {
            allow_privilege_escalation: Some(false),
            read_only_root_filesystem: Some(true),
            ..SecurityContext::default()
        }),
        ..Container::default()
    };
    let template = PodTemplateSpec {
        metadata: Some(ObjectMeta {
            labels: Some(labels(app)),
            annotations: Some(BTreeMap::from([(
                CONFIG_HASH.to_string(),
                config_hash(&spec.config),
            )])),
            ..ObjectMeta::default()
        }),
        spec: Some(PodSpec {
            containers: vec![container],
            volumes: Some(vec![Volume {
                name: "config".into(),
                config_map: Some(ConfigMapVolumeSource {
                    name: format!("{name}-config"),
                    ..ConfigMapVolumeSource::default()
                }),
                ..Volume::default()
            }]),
            security_context: Some(PodSecurityContext {
                run_as_non_root: Some(true),
                run_as_user: Some(10001),
                ..PodSecurityContext::default()
            }),
            ..PodSpec::default()
        }),
    };
    Deployment {
        metadata: child_meta(app, name),
        spec: Some(DeploymentSpec {
            replicas: Some(spec.replicas),
            selector: LabelSelector {
                match_labels: Some(selector(app)),
                ..LabelSelector::default()
            },
            template,
            ..DeploymentSpec::default()
        }),
        ..Deployment::default()
    }
}

pub fn desired_service(app: &App) -> Service {
    Service {
        metadata: child_meta(app, app.name_any()),
        spec: Some(ServiceSpec {
            selector: Some(selector(app)),
            ports: Some(vec![ServicePort {
                name: Some("http".into()),
                port: 80,
                target_port: Some(IntOrString::String("http".into())),
                ..ServicePort::default()
            }]),
            ..ServiceSpec::default()
        }),
        ..Service::default()
    }
}

/// Everything an App should own.
#[derive(Debug, Clone, PartialEq)]
pub struct Desired {
    pub configmap: ConfigMap,
    pub deployment: Deployment,
    pub service: Service,
}

pub fn desired(app: &App) -> Desired {
    Desired {
        configmap: desired_configmap(app),
        deployment: desired_deployment(app),
        service: desired_service(app),
    }
}
