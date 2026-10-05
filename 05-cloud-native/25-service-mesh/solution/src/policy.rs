//! Exercise 6: authorization by workload identity.
//!
//! With mTLS every caller has a verified identity (a SPIFFE id from its
//! certificate), so the mesh can enforce *who may call what*, independent of
//! IP addresses that change with every pod. Deny by default; allow by rule.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// An exact SPIFFE id, or a prefix ending in `*`:
    /// "spiffe://cluster.local/ns/shop/*".
    pub from: String,
    /// Allowed methods; empty = any.
    pub methods: Vec<String>,
    /// Allowed path prefix ("/" = any path).
    pub path_prefix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// No identity at all (no client certificate).
    Unauthenticated,
    Deny,
}

#[derive(Debug, Clone, Default)]
pub struct Policy {
    pub rules: Vec<Rule>,
}

fn identity_matches(pattern: &str, identity: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => identity.starts_with(prefix),
        None => pattern == identity,
    }
}

impl Policy {
    pub fn authorize(&self, identity: Option<&str>, method: &str, path: &str) -> Decision {
        let Some(id) = identity else {
            return Decision::Unauthenticated;
        };
        let allowed = self.rules.iter().any(|r| {
            identity_matches(&r.from, id)
                && (r.methods.is_empty()
                    || r.methods.iter().any(|m| m.eq_ignore_ascii_case(method)))
                && path.starts_with(&r.path_prefix)
        });
        if allowed {
            Decision::Allow
        } else {
            Decision::Deny
        }
    }
}
