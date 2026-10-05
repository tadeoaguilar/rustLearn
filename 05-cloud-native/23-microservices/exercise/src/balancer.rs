//! Bonus: client-side load balancing.
//!
//! With several Inventory instances, the client can spread calls itself:
//! round-robin over the instances it knows (from DNS, Consul, the Kubernetes
//! API...), skipping any that recently failed. That's what gRPC's built-in
//! balancers and service meshes do; here it's spelled out.

use crate::pb::inventory_client::InventoryClient;
use crate::pb::{GetItemRequest, Item};
use crate::server::lazy_channel;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tonic::transport::Channel;
use tonic::{Code, Status};

pub struct Balancer {
    endpoints: Vec<(String, InventoryClient<Channel>)>,
    down_until: Mutex<Vec<Option<Instant>>>,
    next: AtomicUsize,
    cooldown: Duration,
}

impl Balancer {
    pub fn new(urls: &[String], cooldown: Duration) -> Balancer {
        todo!("Bonus")
    }

    /// The next healthy endpoint in round-robin order.
    pub fn pick(&self, now: Instant) -> Option<usize> {
        todo!("Bonus")
    }

    pub fn mark_down(&self, index: usize, now: Instant) {
        todo!("Bonus")
    }

    /// GetItem on some healthy instance; on UNAVAILABLE, mark it down and
    /// try the next. Returns which instance answered.
    pub async fn get_item(&self, sku: &str) -> Result<(String, Item), Status> {
        todo!("Bonus")
    }
}
