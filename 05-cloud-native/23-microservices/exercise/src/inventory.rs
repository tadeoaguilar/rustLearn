//! Exercises 1 and 2: the Inventory service.
//!
//! gRPC errors are a `Status` with a **code** that tells the caller what
//! kind of failure it is -- and so what to do about it:
//!
//! | Code | Meaning | Retry? |
//! |---|---|---|
//! | `InvalidArgument` | the request itself is wrong | no |
//! | `NotFound` | the thing doesn't exist | no |
//! | `FailedPrecondition` | not possible in the current state (out of stock) | not as-is |
//! | `AlreadyExists` | conflicts with something existing | no |
//! | `Unavailable` | try again later | yes, with backoff |
//! | `DeadlineExceeded` | ran out of time | only if idempotent |

use crate::context::{CallContext, parse_grpc_timeout};
use crate::pb::inventory_server::Inventory;
use crate::pb::{
    GetItemRequest, Item, ReleaseRequest, ReleaseResponse, Reservation, ReserveRequest, StockEvent,
    WatchStockRequest,
};
use std::collections::{BTreeMap, HashMap};
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
struct Stock {
    items: BTreeMap<String, Item>,
    /// reservation id -> the reservation, for idempotency and release.
    reservations: HashMap<String, Reservation>,
}

/// (x-request-id, grpc-timeout) as received.
pub type Seen = (Option<String>, Option<Duration>);

/// Cheap to clone: every clone shares the same stock.
#[derive(Clone)]
pub struct InventoryService {
    stock: Arc<Mutex<Stock>>,
    events: broadcast::Sender<StockEvent>,
    /// Test hook: the next N calls fail with UNAVAILABLE.
    fail_next: Arc<AtomicU32>,
    calls: Arc<AtomicU64>,
    /// Test hook: how long each call takes.
    delay: Arc<Mutex<Duration>>,
    /// What the last request carried: (x-request-id, grpc-timeout) -- to
    /// check that the Orders service propagates them.
    last_seen: Arc<Mutex<Option<Seen>>>,
}

impl InventoryService {
    /// `items`: (sku, name, available, price in cents).
    pub fn with_items(items: &[(&str, &str, u32, u64)]) -> InventoryService {
        let items = items
            .iter()
            .map(|&(sku, name, available, price_cents)| {
                (
                    sku.to_string(),
                    Item {
                        sku: sku.into(),
                        name: name.into(),
                        available,
                        price_cents,
                    },
                )
            })
            .collect();
        InventoryService {
            stock: Arc::new(Mutex::new(Stock {
                items,
                reservations: HashMap::new(),
            })),
            events: broadcast::channel(64).0,
            fail_next: Arc::new(AtomicU32::new(0)),
            calls: Arc::new(AtomicU64::new(0)),
            delay: Arc::new(Mutex::new(Duration::ZERO)),
            last_seen: Arc::new(Mutex::new(None)),
        }
    }

    pub fn fail_next(&self, n: u32) {
        self.fail_next.store(n, Ordering::SeqCst);
    }

    /// RPCs received (including failed ones).
    pub fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn available(&self, sku: &str) -> Option<u32> {
        self.stock
            .lock()
            .unwrap()
            .items
            .get(sku)
            .map(|i| i.available)
    }

    pub fn set_delay(&self, delay: Duration) {
        *self.delay.lock().unwrap() = delay;
    }

    /// (x-request-id, grpc-timeout) of the last request received.
    pub fn last_seen(&self) -> Option<Seen> {
        self.last_seen.lock().unwrap().clone()
    }

    /// Bookkeeping and test hooks, at the start of every RPC.
    async fn enter<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let ctx = CallContext::from_request(request);
        let timeout = request
            .metadata()
            .get("grpc-timeout")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_grpc_timeout);
        *self.last_seen.lock().unwrap() = Some((Some(ctx.request_id), timeout));
        let delay = *self.delay.lock().unwrap();
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        let failing = self
            .fail_next
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .is_ok();
        if failing {
            Err(Status::unavailable("inventory is restarting"))
        } else {
            Ok(())
        }
    }

    fn publish(&self, item: &Item) {
        // No subscribers is fine: send only fails when nobody listens.
        let _ = self.events.send(StockEvent {
            sku: item.sku.clone(),
            available: item.available,
        });
    }
}

#[tonic::async_trait]
impl Inventory for InventoryService {
    async fn get_item(&self, request: Request<GetItemRequest>) -> Result<Response<Item>, Status> {
        todo!("Exercise 1")
    }

    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<Reservation>, Status> {
        todo!("Exercise 1")
    }

    async fn release(
        &self,
        request: Request<ReleaseRequest>,
    ) -> Result<Response<ReleaseResponse>, Status> {
        todo!("Exercise 1")
    }

    type WatchStockStream = Pin<Box<dyn Stream<Item = Result<StockEvent, Status>> + Send>>;

    async fn watch_stock(
        &self,
        request: Request<WatchStockRequest>,
    ) -> Result<Response<Self::WatchStockStream>, Status> {
        todo!("Exercise 2")
    }
}
