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
    todo!("Exercise 3")
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
        todo!("Exercise 3")
    }

    /// Time left before the deadline (zero if it passed); None: no deadline.
    pub fn remaining(&self) -> Option<Duration> {
        todo!("Exercise 3")
    }

    /// An outgoing request carrying the request id and the remaining time.
    /// Fails fast if the deadline has already passed.
    pub fn outgoing<T>(&self, message: T) -> Result<Request<T>, Status> {
        todo!("Exercise 3")
    }
}
