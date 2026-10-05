//! Module 25 -- Service Mesh. YOUR WORKSPACE.
//!
//! A service mesh puts a proxy next to every service and moves networking
//! concerns -- retries, timeouts, load balancing, canaries, mTLS,
//! authorization -- out of application code into the proxies. This crate
//! builds those proxy features in Rust, to see what Linkerd and Istio do.
//!
//! | File            | Exercise |
//! |-----------------|----------|
//! | `tcp_proxy.rs`  | 1  an L4 proxy |
//! | `routing.rs`    | 2  L7 routing, hop-by-hop and forwarding headers |
//! | `resilience.rs` | 3  retryable requests and a retry budget |
//! | `split.rs`      | 4  weighted, sticky traffic splits and a canary controller |
//! | `mtls.rs`       | 5  a CA, SPIFFE identities, mutual TLS |
//! | `policy.rs`     | 6  authorization by identity |
//! | `balancer.rs`   | 7  P2C load balancing and outlier ejection |
//! | `proxy.rs`      | 2-4, 7 together: the sidecar |
//! | `manifests.rs`  | bonus: HTTPRoute and Istio policies |
//! | `upstream.rs`   | test upstreams (provided) |

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod balancer;
pub mod manifests;
pub mod mtls;
pub mod policy;
pub mod proxy;
pub mod resilience;
pub mod routing;
pub mod split;
pub mod tcp_proxy;
pub mod upstream;
