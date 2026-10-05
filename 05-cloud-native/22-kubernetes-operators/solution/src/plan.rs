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
        self.configmap.is_none() && self.deployment.is_none() && self.service.is_none()
    }
}

fn change<T>(observed: Option<&T>, diff: impl FnOnce(&T) -> Vec<&'static str>) -> Option<Change> {
    match observed {
        None => Some(Change::Create),
        Some(o) => {
            let fields = diff(o);
            (!fields.is_empty()).then_some(Change::Update(fields))
        }
    }
}

pub fn plan(desired: &Desired, observed: &Observed) -> Plan {
    Plan {
        configmap: change(observed.configmap.as_ref(), |o| {
            diff_configmap(&desired.configmap, o)
        }),
        deployment: change(observed.deployment.as_ref(), |o| {
            diff_deployment(&desired.deployment, o)
        }),
        service: change(observed.service.as_ref(), |o| {
            diff_service(&desired.service, o)
        }),
    }
}

fn diff_configmap(d: &ConfigMap, o: &ConfigMap) -> Vec<&'static str> {
    // An empty map and a missing one are the same to Kubernetes.
    let norm = |c: &ConfigMap| c.data.clone().unwrap_or_default();
    if norm(d) != norm(o) {
        vec!["data"]
    } else {
        vec![]
    }
}

fn container(d: &Deployment) -> Option<&Container> {
    d.spec.as_ref()?.template.spec.as_ref()?.containers.first()
}

/// (path, port) of an HTTP probe; timeouts and thresholds are server defaults.
fn probe_key(p: &Option<Probe>) -> Option<(Option<String>, String)> {
    let get = p.as_ref()?.http_get.as_ref()?;
    Some((get.path.clone(), format!("{:?}", get.port)))
}

fn diff_deployment(d: &Deployment, o: &Deployment) -> Vec<&'static str> {
    let mut fields = Vec::new();
    let replicas = |x: &Deployment| x.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);
    if replicas(d) != replicas(o) {
        fields.push("replicas");
    }
    let (dc, oc) = (container(d), container(o));
    if dc.and_then(|c| c.image.clone()) != oc.and_then(|c| c.image.clone()) {
        fields.push("image");
    }
    let env = |c: Option<&Container>| -> BTreeMap<String, Option<String>> {
        c.and_then(|c| c.env.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.name, e.value))
            .collect()
    };
    if env(dc) != env(oc) {
        fields.push("env");
    }
    let ports = |c: Option<&Container>| -> Vec<(i32, Option<String>)> {
        c.and_then(|c| c.ports.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.container_port, p.name))
            .collect()
    };
    if ports(dc) != ports(oc) {
        fields.push("ports");
    }
    let probes = |c: Option<&Container>| {
        c.map(|c| (probe_key(&c.liveness_probe), probe_key(&c.readiness_probe)))
    };
    if probes(dc) != probes(oc) {
        fields.push("probes");
    }
    let hash = |x: &Deployment| {
        x.spec
            .as_ref()?
            .template
            .metadata
            .as_ref()?
            .annotations
            .as_ref()?
            .get(CONFIG_HASH)
            .cloned()
    };
    if hash(d) != hash(o) {
        fields.push("config");
    }
    fields
}

fn diff_service(d: &Service, o: &Service) -> Vec<&'static str> {
    let mut fields = Vec::new();
    let selector = |s: &Service| {
        s.spec
            .as_ref()
            .and_then(|s| s.selector.clone())
            .unwrap_or_default()
    };
    if selector(d) != selector(o) {
        fields.push("selector");
    }
    let ports = |s: &Service| -> Vec<(Option<String>, i32, String)> {
        s.spec
            .as_ref()
            .and_then(|s| s.ports.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.name, p.port, format!("{:?}", p.target_port)))
            .collect()
    };
    if ports(d) != ports(o) {
        fields.push("ports");
    }
    fields
}
