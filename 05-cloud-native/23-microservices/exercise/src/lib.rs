//! Module 23 -- Microservices. YOUR WORKSPACE.
//!
//! Two gRPC services generated from `proto/shop.proto`: Inventory owns stock,
//! Orders places orders by calling Inventory and a payment provider.
//!
//! | File           | Exercise |
//! |----------------|----------|
//! | `inventory.rs` | 1, 2  a gRPC service: status codes, idempotency, server streaming |
//! | `context.rs`   | 3  request ids and deadlines across calls |
//! | `retry.rs`     | 4  retries with exponential backoff and jitter |
//! | `breaker.rs`   | 5  a circuit breaker |
//! | `events.rs`    | 6  at-least-once events and idempotent consumers |
//! | `orders.rs`    | 7  the Orders service: a saga with compensation |
//! | `balancer.rs`  | bonus: client-side load balancing |
//! | `server.rs`    | starting servers on local ports (provided) |

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod pb {
    tonic::include_proto!("shop.v1");
}
pub mod balancer;
pub mod breaker;
pub mod context;
pub mod events;
pub mod inventory;
pub mod orders;
pub mod retry;
pub mod server;
