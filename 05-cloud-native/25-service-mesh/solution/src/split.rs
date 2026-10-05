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
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Picks a cluster in proportion to the weights. With a `sticky_key` the
/// choice depends only on it; otherwise on `roll` (a random number).
/// Zero-weight clusters are never picked. None if all weights are zero.
pub fn pick_weighted<'a>(
    weights: &'a [(String, u32)],
    sticky_key: Option<&str>,
    roll: u64,
) -> Option<&'a str> {
    let total: u64 = weights.iter().map(|(_, w)| u64::from(*w)).sum();
    if total == 0 {
        return None;
    }
    let mut point = sticky_key.map_or(roll, fnv1a) % total;
    for (name, w) in weights {
        let w = u64::from(*w);
        if point < w {
            return Some(name);
        }
        point -= w;
    }
    unreachable!("point < total")
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
        CanaryController {
            steps,
            step: 0,
            max_error_rate,
            min_requests,
            rolled_back: false,
        }
    }

    /// The canary's current share of traffic, in percent.
    pub fn weight(&self) -> u32 {
        if self.rolled_back {
            0
        } else {
            self.steps.get(self.step).copied().unwrap_or(100)
        }
    }

    /// Weights for `pick_weighted`.
    pub fn weights(&self, stable: &str, canary: &str) -> Vec<(String, u32)> {
        vec![
            (stable.to_string(), 100 - self.weight()),
            (canary.to_string(), self.weight()),
        ]
    }

    /// Judges the canary's traffic since the last step.
    pub fn evaluate(&mut self, canary_requests: u64, canary_errors: u64) -> Decision {
        if self.rolled_back {
            return Decision::Rollback;
        }
        if canary_requests > 0
            && canary_errors as f64 / canary_requests as f64 > self.max_error_rate
        {
            self.rolled_back = true;
            return Decision::Rollback; // don't wait for min_requests to act on a bad canary
        }
        if canary_requests < self.min_requests {
            return Decision::Hold;
        }
        if self.step + 1 >= self.steps.len() {
            self.step = self.steps.len();
            return Decision::Complete;
        }
        self.step += 1;
        if self.weight() >= 100 {
            Decision::Complete
        } else {
            Decision::Promote(self.weight())
        }
    }
}
