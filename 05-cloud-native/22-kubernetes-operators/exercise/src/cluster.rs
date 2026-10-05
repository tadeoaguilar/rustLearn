//! The operator talks to the cluster through this trait, so the same
//! reconcile code runs against a real API server (`controller.rs`) and
//! against `FakeCluster`, an in-memory API server for tests.
//!
//! PROVIDED in the exercise crate: read it to see what a real API server does
//! with your writes -- it fills in defaults, bumps `metadata.generation`
//! when the spec changes, keeps objects with finalizers around until the
//! finalizers are removed, and garbage-collects objects whose owner is gone.

use k8s_openapi::NamespaceResourceScope;
use kube::{Resource, ResourceExt};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Mutex;

/// Anything the operator reads or writes: namespaced, typed, serialisable.
pub trait Object:
    Resource<DynamicType = (), Scope = NamespaceResourceScope>
    + Clone
    + Serialize
    + DeserializeOwned
    + Debug
    + Send
    + Sync
    + 'static
{
}

impl<T> Object for T where
    T: Resource<DynamicType = (), Scope = NamespaceResourceScope>
        + Clone
        + Serialize
        + DeserializeOwned
        + Debug
        + Send
        + Sync
        + 'static
{
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClusterError {
    #[error("{0} not found")]
    NotFound(String),
    #[error("API error: {0}")]
    Api(String),
}

pub trait Cluster: Send + Sync {
    fn get<K: Object>(
        &self,
        namespace: &str,
        name: &str,
    ) -> impl Future<Output = Result<Option<K>, ClusterError>> + Send;
    /// Every object of the kind, in all namespaces.
    fn list<K: Object>(&self) -> impl Future<Output = Result<Vec<K>, ClusterError>> + Send;
    /// Server-side apply: create or update everything but the status.
    fn apply<K: Object>(&self, obj: &K) -> impl Future<Output = Result<K, ClusterError>> + Send;
    /// Writes only the status subresource.
    fn apply_status<K: Object>(
        &self,
        obj: &K,
    ) -> impl Future<Output = Result<(), ClusterError>> + Send;
    fn set_finalizers<K: Object>(
        &self,
        namespace: &str,
        name: &str,
        finalizers: Vec<String>,
    ) -> impl Future<Output = Result<(), ClusterError>> + Send;
    /// Idempotent: deleting something that's gone is fine.
    fn delete<K: Object>(
        &self,
        namespace: &str,
        name: &str,
    ) -> impl Future<Output = Result<(), ClusterError>> + Send;
}

type Key = (String, String, String); // kind, namespace, name

#[derive(Default)]
struct Inner {
    objects: BTreeMap<Key, Value>,
    next_uid: u64,
    next_version: u64,
    writes: u64,
    fail_next: u32,
}

/// An in-memory API server. Every successful write that changes something
/// counts in `writes()`; no-op writes don't (like the real server).
#[derive(Default)]
pub struct FakeCluster {
    inner: Mutex<Inner>,
}

fn key_of<K: Object>(namespace: &str, name: &str) -> Key {
    (
        K::kind(&()).to_string(),
        namespace.to_string(),
        name.to_string(),
    )
}

fn meta_mut(v: &mut Value) -> &mut serde_json::Map<String, Value> {
    if !v["metadata"].is_object() {
        v["metadata"] = json!({});
    }
    v["metadata"].as_object_mut().unwrap()
}

/// What the API server fills in when you don't (a small but real subset).
fn apply_defaults(kind: &str, v: &mut Value) {
    fn set_default(v: &mut Value, key: &str, default: Value) {
        if v.is_object() && v.get(key).is_none_or(Value::is_null) {
            v[key] = default;
        }
    }
    fn default_probe(p: &mut Value) {
        if p.is_object() {
            for (k, d) in [
                ("timeoutSeconds", 1),
                ("successThreshold", 1),
                ("failureThreshold", 3),
            ] {
                set_default(p, k, json!(d));
            }
            if p["httpGet"].is_object() {
                set_default(&mut p["httpGet"], "scheme", json!("HTTP"));
            }
        }
    }
    match kind {
        "Deployment" => {
            let spec = &mut v["spec"];
            set_default(spec, "replicas", json!(1));
            set_default(spec, "revisionHistoryLimit", json!(10));
            set_default(spec, "progressDeadlineSeconds", json!(600));
            set_default(
                spec,
                "strategy",
                json!({"type": "RollingUpdate", "rollingUpdate": {"maxSurge": "25%", "maxUnavailable": "25%"}}),
            );
            if let Some(containers) = spec["template"]["spec"]["containers"].as_array_mut() {
                for c in containers {
                    set_default(c, "imagePullPolicy", json!("IfNotPresent"));
                    set_default(c, "terminationMessagePath", json!("/dev/termination-log"));
                    if let Some(ports) = c["ports"].as_array_mut() {
                        for p in ports {
                            set_default(p, "protocol", json!("TCP"));
                        }
                    }
                    default_probe(&mut c["livenessProbe"]);
                    default_probe(&mut c["readinessProbe"]);
                }
            }
            set_default(
                &mut spec["template"]["spec"],
                "restartPolicy",
                json!("Always"),
            );
        }
        "Service" => {
            let spec = &mut v["spec"];
            set_default(spec, "type", json!("ClusterIP"));
            set_default(spec, "sessionAffinity", json!("None"));
            if let Some(ports) = spec["ports"].as_array_mut() {
                for p in ports {
                    set_default(p, "protocol", json!("TCP"));
                }
            }
        }
        _ => {}
    }
}

/// The parts of an object that count as its spec (a change bumps generation).
fn spec_of(v: &Value) -> Value {
    json!({ "spec": v.get("spec"), "data": v.get("data") })
}

impl FakeCluster {
    pub fn new() -> FakeCluster {
        FakeCluster::default()
    }

    /// Writes that changed something, since the start.
    pub fn writes(&self) -> u64 {
        self.inner.lock().unwrap().writes
    }

    /// The next `n` API calls fail -- a flaky API server or network.
    pub fn fail_next(&self, n: u32) {
        self.inner.lock().unwrap().fail_next = n;
    }

    pub fn count<K: Object>(&self) -> usize {
        let kind = K::kind(&()).to_string();
        self.inner
            .lock()
            .unwrap()
            .objects
            .keys()
            .filter(|(k, _, _)| *k == kind)
            .count()
    }

    fn check_failure(inner: &mut Inner) -> Result<(), ClusterError> {
        if inner.fail_next > 0 {
            inner.fail_next -= 1;
            return Err(ClusterError::Api("injected failure".into()));
        }
        Ok(())
    }

    fn bump(inner: &mut Inner, v: &mut Value) {
        inner.next_version += 1;
        meta_mut(v).insert(
            "resourceVersion".into(),
            json!(inner.next_version.to_string()),
        );
        inner.writes += 1;
    }

    /// A user's `kubectl edit`: change any field of a stored object.
    pub fn edit<K: Object>(
        &self,
        namespace: &str,
        name: &str,
        f: impl FnOnce(&mut K),
    ) -> Result<(), ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        let key = key_of::<K>(namespace, name);
        let old = inner
            .objects
            .get(&key)
            .cloned()
            .ok_or_else(|| ClusterError::NotFound(name.into()))?;
        let mut obj: K =
            serde_json::from_value(old.clone()).map_err(|e| ClusterError::Api(e.to_string()))?;
        f(&mut obj);
        let mut new = serde_json::to_value(&obj).unwrap();
        apply_defaults(&key.0, &mut new);
        if spec_of(&new) != spec_of(&old) {
            let generation = old["metadata"]["generation"].as_i64().unwrap_or(1) + 1;
            meta_mut(&mut new).insert("generation".into(), json!(generation));
        }
        if new != old {
            Self::bump(&mut inner, &mut new);
            inner.objects.insert(key, new);
        }
        Ok(())
    }

    /// Removes an object and, recursively, everything it owns.
    fn remove_cascading(inner: &mut Inner, key: &Key) {
        let Some(removed) = inner.objects.remove(key) else {
            return;
        };
        inner.writes += 1;
        let Some(uid) = removed["metadata"]["uid"].as_str().map(str::to_string) else {
            return;
        };
        let owned: Vec<Key> = inner
            .objects
            .iter()
            .filter(|(_, v)| {
                v["metadata"]["ownerReferences"]
                    .as_array()
                    .is_some_and(|refs| refs.iter().any(|r| r["uid"] == uid.as_str()))
            })
            .map(|(k, _)| k.clone())
            .collect();
        for k in owned {
            Self::delete_key(inner, &k);
        }
    }

    /// Deletion as the API server does it: objects with finalizers only get
    /// a deletionTimestamp; the rest go (with their dependents).
    fn delete_key(inner: &mut Inner, key: &Key) {
        let Some(v) = inner.objects.get(key) else {
            return;
        };
        let has_finalizers = v["metadata"]["finalizers"]
            .as_array()
            .is_some_and(|f| !f.is_empty());
        if has_finalizers {
            if v["metadata"]["deletionTimestamp"].is_null() {
                let mut v = v.clone();
                meta_mut(&mut v).insert("deletionTimestamp".into(), json!("2026-01-01T00:00:00Z"));
                Self::bump(inner, &mut v);
                inner.objects.insert(key.clone(), v);
            }
        } else {
            Self::remove_cascading(inner, key);
        }
    }

    /// The built-in Deployment controller, one step: pods become ready two
    /// at a time; an image containing "broken" never becomes ready.
    pub fn run_builtin_controllers(&self) {
        let mut inner = self.inner.lock().unwrap();
        let keys: Vec<Key> = inner
            .objects
            .keys()
            .filter(|(k, _, _)| k == "Deployment")
            .cloned()
            .collect();
        for key in keys {
            let mut v = inner.objects[&key].clone();
            let desired = v["spec"]["replicas"].as_i64().unwrap_or(1);
            let generation = v["metadata"]["generation"].clone();
            let image = v["spec"]["template"]["spec"]["containers"][0]["image"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let old_ready = v["status"]["readyReplicas"].as_i64().unwrap_or(0);
            let rolled = v["status"]["observedGeneration"] == generation;
            let broken = image.contains("broken");
            // A new generation restarts the rollout from whatever is ready.
            let ready = if broken {
                0
            } else if old_ready < desired || !rolled {
                (old_ready.min(desired) + 2).min(desired)
            } else {
                desired
            };
            let conditions = if broken {
                json!([{"type": "Progressing", "status": "False", "reason": "ProgressDeadlineExceeded"}])
            } else {
                json!([{"type": "Available", "status": if ready == desired { "True" } else { "False" }}])
            };
            let status = json!({
                "observedGeneration": generation,
                "replicas": desired,
                "updatedReplicas": if broken { 0 } else { desired },
                "readyReplicas": ready,
                "availableReplicas": ready,
                "conditions": conditions,
            });
            if v["status"] != status {
                v["status"] = status;
                Self::bump(&mut inner, &mut v);
                inner.objects.insert(key, v);
            }
        }
    }
}

impl Cluster for FakeCluster {
    async fn get<K: Object>(&self, namespace: &str, name: &str) -> Result<Option<K>, ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        Self::check_failure(&mut inner)?;
        inner
            .objects
            .get(&key_of::<K>(namespace, name))
            .map(|v| {
                serde_json::from_value(v.clone()).map_err(|e| ClusterError::Api(e.to_string()))
            })
            .transpose()
    }

    async fn list<K: Object>(&self) -> Result<Vec<K>, ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        Self::check_failure(&mut inner)?;
        let kind = K::kind(&()).to_string();
        inner
            .objects
            .iter()
            .filter(|((k, _, _), _)| *k == kind)
            .map(|(_, v)| {
                serde_json::from_value(v.clone()).map_err(|e| ClusterError::Api(e.to_string()))
            })
            .collect()
    }

    async fn apply<K: Object>(&self, obj: &K) -> Result<K, ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        Self::check_failure(&mut inner)?;
        let namespace = obj.namespace().unwrap_or_else(|| "default".into());
        let name = obj.name_any();
        if name.is_empty() {
            return Err(ClusterError::Api("metadata.name is required".into()));
        }
        let key = key_of::<K>(&namespace, &name);
        let mut new = serde_json::to_value(obj).map_err(|e| ClusterError::Api(e.to_string()))?;
        if let Some(o) = new.as_object_mut() {
            o.remove("status"); // apply never writes status
        }
        let meta = meta_mut(&mut new);
        meta.insert("namespace".into(), json!(namespace));
        for server_field in [
            "uid",
            "resourceVersion",
            "generation",
            "creationTimestamp",
            "deletionTimestamp",
            "managedFields",
        ] {
            meta.remove(server_field);
        }
        apply_defaults(&key.0, &mut new);

        let stored = match inner.objects.get(&key).cloned() {
            None => {
                inner.next_uid += 1;
                let uid = format!("uid-{}", inner.next_uid);
                let meta = meta_mut(&mut new);
                meta.insert("uid".into(), json!(uid));
                meta.insert("generation".into(), json!(1));
                meta.insert("creationTimestamp".into(), json!("2026-01-01T00:00:00Z"));
                Self::bump(&mut inner, &mut new);
                new
            }
            Some(old) => {
                for field in [
                    "uid",
                    "creationTimestamp",
                    "deletionTimestamp",
                    "generation",
                    "resourceVersion",
                ] {
                    if let Some(value) = old["metadata"].get(field) {
                        meta_mut(&mut new).insert(field.into(), value.clone());
                    }
                }
                // Finalizers belong to whoever added them: keep them unless this apply sets its own.
                if new["metadata"].get("finalizers").is_none()
                    && !old["metadata"]["finalizers"].is_null()
                {
                    meta_mut(&mut new)
                        .insert("finalizers".into(), old["metadata"]["finalizers"].clone());
                }
                if !old["status"].is_null() {
                    new["status"] = old["status"].clone();
                }
                if new == old {
                    old // nothing changed: no write
                } else {
                    if spec_of(&new) != spec_of(&old) {
                        let generation = old["metadata"]["generation"].as_i64().unwrap_or(1) + 1;
                        meta_mut(&mut new).insert("generation".into(), json!(generation));
                    }
                    Self::bump(&mut inner, &mut new);
                    new
                }
            }
        };
        inner.objects.insert(key, stored.clone());
        serde_json::from_value(stored).map_err(|e| ClusterError::Api(e.to_string()))
    }

    async fn apply_status<K: Object>(&self, obj: &K) -> Result<(), ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        Self::check_failure(&mut inner)?;
        let key = key_of::<K>(
            &obj.namespace().unwrap_or_else(|| "default".into()),
            &obj.name_any(),
        );
        let mut stored = inner
            .objects
            .get(&key)
            .cloned()
            .ok_or_else(|| ClusterError::NotFound(key.2.clone()))?;
        let status =
            serde_json::to_value(obj).map_err(|e| ClusterError::Api(e.to_string()))?["status"]
                .clone();
        if stored["status"] != status {
            stored["status"] = status;
            Self::bump(&mut inner, &mut stored);
            inner.objects.insert(key, stored);
        }
        Ok(())
    }

    async fn set_finalizers<K: Object>(
        &self,
        namespace: &str,
        name: &str,
        finalizers: Vec<String>,
    ) -> Result<(), ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        Self::check_failure(&mut inner)?;
        let key = key_of::<K>(namespace, name);
        let mut stored = inner
            .objects
            .get(&key)
            .cloned()
            .ok_or_else(|| ClusterError::NotFound(name.into()))?;
        let value = if finalizers.is_empty() {
            Value::Null
        } else {
            json!(finalizers)
        };
        if stored["metadata"]["finalizers"] != value {
            if value.is_null() {
                meta_mut(&mut stored).remove("finalizers");
            } else {
                meta_mut(&mut stored).insert("finalizers".into(), value);
            }
            Self::bump(&mut inner, &mut stored);
            let deleting = !stored["metadata"]["deletionTimestamp"].is_null();
            inner.objects.insert(key.clone(), stored);
            if deleting && finalizers.is_empty() {
                Self::remove_cascading(&mut inner, &key); // the last finalizer is gone: delete for real
            }
        }
        Ok(())
    }

    async fn delete<K: Object>(&self, namespace: &str, name: &str) -> Result<(), ClusterError> {
        let mut inner = self.inner.lock().unwrap();
        Self::check_failure(&mut inner)?;
        Self::delete_key(&mut inner, &key_of::<K>(namespace, name));
        Ok(())
    }
}
