//! Exercise 4: W3C Trace Context.
//!
//! For a trace to cross services, each request carries the trace's identity
//! in a standard header (https://www.w3.org/TR/trace-context/):
//!
//! ```text
//! traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
//!              ^  ^ trace id: 16 bytes             ^ parent span id   ^ flags (01 = sampled)
//!              version
//! ```
//!
//! Every service, every language, every vendor (OpenTelemetry, Jaeger,
//! Datadog, cloud load balancers) understands it.

use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceParent {
    pub trace_id: u128,
    /// The id of the span that sent this request: the parent of whatever
    /// the receiver creates.
    pub span_id: u64,
    pub sampled: bool,
}

fn hex_field<const N: usize>(s: &str) -> Option<&str> {
    todo!("Exercise 4")
}

impl TraceParent {
    /// Parses a version-00 header. Invalid -> None: start a new trace.
    pub fn parse(header: &str) -> Option<TraceParent> {
        todo!("Exercise 4")
    }

    pub fn header(&self) -> String {
        todo!("Exercise 4")
    }

    pub fn new_root(sampled: bool) -> TraceParent {
        todo!("Exercise 4")
    }

    /// Same trace, a new span id: for the next hop.
    pub fn child(&self) -> TraceParent {
        todo!("Exercise 4")
    }

    pub fn trace_id_hex(&self) -> String {
        todo!("Exercise 4")
    }

    pub fn span_id_hex(&self) -> String {
        todo!("Exercise 4")
    }
}
