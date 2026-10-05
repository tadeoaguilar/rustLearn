//! Exercise 3: carrying context across services.
//!
//! One user request becomes a chain of calls: client -> Orders -> Inventory.
//! Two things must travel along it:
//!
//! - a **request id**, so logs from every service can be joined up;
//! - the **deadline**. If the client gives up after 2 s, Inventory working
//!   on a 5 s query is wasted -- worse, under load it piles up work nobody
//!   waits for. gRPC sends the remaining time in the `grpc-timeout` header;
//!   each hop should pass on whatever is *left*.

use rand::Rng;
use std::time::{Duration, Instant};
use tonic::{Request, Status};

pub const REQUEST_ID: &str = "x-request-id";

/// Parses a `grpc-timeout` value: up to 8 digits and a unit --
/// H(ours) M(inutes) S(econds) m(illis) u(micros) n(anos).
pub fn parse_grpc_timeout(value: &str) -> Option<Duration> {
    let unit = value.chars().last()?;
    let digits = &value[..value.len() - unit.len_utf8()];
    if digits.is_empty() || digits.len() > 8 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u64 = digits.parse().ok()?;
    Some(match unit {
        'H' => Duration::from_secs(n * 3600),
        'M' => Duration::from_secs(n * 60),
        'S' => Duration::from_secs(n),
        'm' => Duration::from_millis(n),
        'u' => Duration::from_micros(n),
        'n' => Duration::from_nanos(n),
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallContext {
    pub request_id: String,
    pub deadline: Option<Instant>,
}

impl CallContext {
    /// From an incoming request: its request id (or a new one) and its
    /// deadline (now + grpc-timeout, if the caller set one).
    pub fn from_request<T>(request: &Request<T>) -> CallContext {
        let meta = request.metadata();
        let request_id = meta
            .get(REQUEST_ID)
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("req-{:016x}", rand::thread_rng().r#gen::<u64>()));
        let deadline = meta
            .get("grpc-timeout")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_grpc_timeout)
            .map(|t| Instant::now() + t);
        CallContext {
            request_id,
            deadline,
        }
    }

    /// Time left before the deadline (zero if it passed); None: no deadline.
    pub fn remaining(&self) -> Option<Duration> {
        self.deadline
            .map(|d| d.saturating_duration_since(Instant::now()))
    }

    /// An outgoing request carrying the request id and the remaining time.
    /// Fails fast if the deadline has already passed.
    pub fn outgoing<T>(&self, message: T) -> Result<Request<T>, Status> {
        let mut request = Request::new(message);
        if let Ok(value) = self.request_id.parse() {
            request.metadata_mut().insert(REQUEST_ID, value);
        }
        if let Some(left) = self.remaining() {
            if left.is_zero() {
                return Err(Status::deadline_exceeded(
                    "deadline passed before calling downstream",
                ));
            }
            request.set_timeout(left);
        }
        Ok(request)
    }
}
