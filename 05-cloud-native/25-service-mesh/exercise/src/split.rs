//! Exercise 4: traffic splitting and progressive delivery.
//!
//! A canary release sends a small share of traffic to the new version and
//! watches it. Two details matter:
//!
//! - **stickiness**: the same user should see the same version on every
//!   request, so pick by a hash of a stable key, not at random;
//! - **automation**: step the weight up while the canary is healthy, roll
//!   back as soon as it isn't (Flagger and Argo Rollouts do this).

/// FNV-1a: a small, stable hash (std's `DefaultHasher` may change between
/// Rust versions, and must not for routing).
pub fn fnv1a(key: &str) -> u64 {
    todo!("Exercise 4")
}

/// Picks a cluster in proportion to the weights. With a `sticky_key` the
/// choice depends only on it; otherwise on `roll` (a random number).
/// Zero-weight clusters are never picked. None if all weights are zero.
pub fn pick_weighted<'a>(
    weights: &'a [(String, u32)],
    sticky_key: Option<&str>,
    roll: u64,
) -> Option<&'a str> {
    todo!("Exercise 4")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Not enough traffic yet to judge.
    Hold,
    /// Healthy: move to this canary weight (percent).
    Promote(u32),
    /// The canary is now the only version.
    Complete,
    /// Unhealthy: send everything back to stable.
    Rollback,
}

#[derive(Debug, Clone)]
pub struct CanaryController {
    steps: Vec<u32>,
    step: usize,
    max_error_rate: f64,
    min_requests: u64,
    rolled_back: bool,
}

impl CanaryController {
    /// `steps`: canary weights in percent, e.g. [5, 25, 50, 100].
    pub fn new(steps: Vec<u32>, max_error_rate: f64, min_requests: u64) -> CanaryController {
        todo!("Exercise 4")
    }

    /// The canary's current share of traffic, in percent.
    pub fn weight(&self) -> u32 {
        todo!("Exercise 4")
    }

    /// Weights for `pick_weighted`.
    pub fn weights(&self, stable: &str, canary: &str) -> Vec<(String, u32)> {
        todo!("Exercise 4")
    }

    /// Judges the canary's traffic since the last step.
    pub fn evaluate(&mut self, canary_requests: u64, canary_errors: u64) -> Decision {
        todo!("Exercise 4")
    }
}
