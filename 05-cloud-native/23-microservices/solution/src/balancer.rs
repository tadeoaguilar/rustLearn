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
        Balancer {
            endpoints: urls
                .iter()
                .map(|u| (u.clone(), InventoryClient::new(lazy_channel(u))))
                .collect(),
            down_until: Mutex::new(vec![None; urls.len()]),
            next: AtomicUsize::new(0),
            cooldown,
        }
    }

    /// The next healthy endpoint in round-robin order.
    pub fn pick(&self, now: Instant) -> Option<usize> {
        let down = self.down_until.lock().unwrap();
        let n = self.endpoints.len();
        (0..n)
            .map(|_| self.next.fetch_add(1, Ordering::Relaxed) % n)
            .find(|&i| down[i].is_none_or(|until| until <= now))
    }

    pub fn mark_down(&self, index: usize, now: Instant) {
        self.down_until.lock().unwrap()[index] = Some(now + self.cooldown);
    }

    /// GetItem on some healthy instance; on UNAVAILABLE, mark it down and
    /// try the next. Returns which instance answered.
    pub async fn get_item(&self, sku: &str) -> Result<(String, Item), Status> {
        for _ in 0..self.endpoints.len() {
            let Some(i) = self.pick(Instant::now()) else {
                break;
            };
            let (url, client) = &self.endpoints[i];
            match client
                .clone()
                .get_item(GetItemRequest { sku: sku.into() })
                .await
            {
                Ok(r) => return Ok((url.clone(), r.into_inner())),
                Err(s) if s.code() == Code::Unavailable => self.mark_down(i, Instant::now()),
                Err(s) => return Err(s),
            }
        }
        Err(Status::unavailable("no healthy inventory instance"))
    }
}
