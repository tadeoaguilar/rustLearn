//! Bonus: the same policies, as mesh configuration.
//!
//! What this module's code does by hand, Linkerd and Istio do from
//! Kubernetes resources. Generating them from the same data as the Rust
//! controller keeps them in sync.

use crate::policy::Policy;
use serde_json::{Value, json};

/// A Gateway API HTTPRoute splitting traffic between two Services -- the
/// resource Linkerd (and Istio) use for canaries.
pub fn http_route(
    namespace: &str,
    service: &str,
    stable: &str,
    canary: &str,
    canary_weight: u32,
) -> Value {
    json!({
        "apiVersion": "gateway.networking.k8s.io/v1",
        "kind": "HTTPRoute",
        "metadata": { "name": format!("{service}-split"), "namespace": namespace },
        "spec": {
            "parentRefs": [{ "name": service, "kind": "Service", "group": "core", "port": 80 }],
            "rules": [{
                "backendRefs": [
                    { "name": stable, "port": 80, "weight": 100 - canary_weight },
                    { "name": canary, "port": 80, "weight": canary_weight },
                ],
            }],
        },
    })
}

/// An Istio AuthorizationPolicy with one ALLOW rule per policy rule. Istio
/// principals are SPIFFE ids without the "spiffe://" scheme.
pub fn authorization_policy(namespace: &str, app: &str, policy: &Policy) -> Value {
    let rules: Vec<Value> = policy
        .rules
        .iter()
        .map(|r| {
            let principal = r.from.trim_start_matches("spiffe://");
            let mut to = json!({ "paths": [format!("{}*", r.path_prefix)] });
            if !r.methods.is_empty() {
                to["methods"] = json!(r.methods);
            }
            json!({ "from": [{ "source": { "principals": [principal] } }], "to": [{ "operation": to }] })
        })
        .collect();
    json!({
        "apiVersion": "security.istio.io/v1",
        "kind": "AuthorizationPolicy",
        "metadata": { "name": format!("{app}-allow"), "namespace": namespace },
        "spec": { "selector": { "matchLabels": { "app": app } }, "action": "ALLOW", "rules": rules },
    })
}

/// Istio PeerAuthentication: mTLS required in the namespace.
pub fn strict_mtls(namespace: &str) -> Value {
    json!({
        "apiVersion": "security.istio.io/v1",
        "kind": "PeerAuthentication",
        "metadata": { "name": "default", "namespace": namespace },
        "spec": { "mtls": { "mode": "STRICT" } },
    })
}
