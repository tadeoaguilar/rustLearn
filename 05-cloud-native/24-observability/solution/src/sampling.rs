//! Bonus: sampling.
//!
//! Recording every span of every request is expensive at scale. Head-based
//! sampling decides once, at the start of a trace, and the decision travels
//! in the `traceparent` flags so every service keeps or drops the *same*
//! traces -- a trace with holes in it is useless.

use crate::tracecontext::TraceParent;

/// Parent-based, else trace-id ratio: a trace with a parent follows the
/// parent's decision; a new trace is sampled if the low 64 bits of its id
/// fall in the lowest `ratio` of the range -- the same answer in every
/// service for the same trace id, with no coordination.
pub fn should_sample(parent: Option<&TraceParent>, trace_id: u128, ratio: f64) -> bool {
    if let Some(p) = parent {
        return p.sampled;
    }
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio >= 1.0 {
        return true;
    }
    let threshold = (ratio * u64::MAX as f64) as u64;
    (trace_id as u64) < threshold
}
