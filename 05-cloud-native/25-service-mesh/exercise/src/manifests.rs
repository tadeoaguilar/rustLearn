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
    todo!("Bonus")
}

/// An Istio AuthorizationPolicy with one ALLOW rule per policy rule. Istio
/// principals are SPIFFE ids without the "spiffe://" scheme.
pub fn authorization_policy(namespace: &str, app: &str, policy: &Policy) -> Value {
    todo!("Bonus")
}

/// Istio PeerAuthentication: mTLS required in the namespace.
pub fn strict_mtls(namespace: &str) -> Value {
    todo!("Bonus")
}
