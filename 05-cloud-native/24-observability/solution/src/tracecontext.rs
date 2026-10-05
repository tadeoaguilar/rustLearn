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
    (s.len() == N
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
    .then_some(s)
}

impl TraceParent {
    /// Parses a version-00 header. Invalid -> None: start a new trace.
    pub fn parse(header: &str) -> Option<TraceParent> {
        let parts: Vec<&str> = header.trim().split('-').collect();
        let [version, trace, span, flags] = parts.as_slice() else {
            return None;
        };
        if hex_field::<2>(version)? != "00" {
            return None; // "ff" is forbidden; future versions may change the format
        }
        let trace_id = u128::from_str_radix(hex_field::<32>(trace)?, 16).ok()?;
        let span_id = u64::from_str_radix(hex_field::<16>(span)?, 16).ok()?;
        let flags = u8::from_str_radix(hex_field::<2>(flags)?, 16).ok()?;
        if trace_id == 0 || span_id == 0 {
            return None; // all-zero ids are invalid
        }
        Some(TraceParent {
            trace_id,
            span_id,
            sampled: flags & 1 == 1,
        })
    }

    pub fn header(&self) -> String {
        format!(
            "00-{:032x}-{:016x}-{:02x}",
            self.trace_id,
            self.span_id,
            u8::from(self.sampled)
        )
    }

    pub fn new_root(sampled: bool) -> TraceParent {
        let mut rng = rand::thread_rng();
        TraceParent {
            trace_id: rng.gen_range(1..=u128::MAX),
            span_id: rng.gen_range(1..=u64::MAX),
            sampled,
        }
    }

    /// Same trace, a new span id: for the next hop.
    pub fn child(&self) -> TraceParent {
        TraceParent {
            span_id: rand::thread_rng().gen_range(1..=u64::MAX),
            ..*self
        }
    }

    pub fn trace_id_hex(&self) -> String {
        format!("{:032x}", self.trace_id)
    }

    pub fn span_id_hex(&self) -> String {
        format!("{:016x}", self.span_id)
    }
}
