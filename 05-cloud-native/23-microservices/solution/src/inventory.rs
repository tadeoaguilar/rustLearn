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
        self.enter(&request).await?;
        let sku = request.into_inner().sku;
        let stock = self.stock.lock().unwrap();
        match stock.items.get(&sku) {
            Some(item) => Ok(Response::new(item.clone())),
            None => Err(Status::not_found(format!("no item with sku {sku:?}"))),
        }
    }

    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<Reservation>, Status> {
        self.enter(&request).await?;
        let req = request.into_inner();
        if req.reservation_id.is_empty() {
            return Err(Status::invalid_argument("reservation_id is required"));
        }
        if req.quantity == 0 {
            return Err(Status::invalid_argument("quantity must be at least 1"));
        }
        let mut stock = self.stock.lock().unwrap();
        // Idempotency: the same request again returns the same answer.
        if let Some(existing) = stock.reservations.get(&req.reservation_id) {
            return if existing.sku == req.sku && existing.quantity == req.quantity {
                Ok(Response::new(existing.clone()))
            } else {
                Err(Status::already_exists(format!(
                    "reservation {} exists with different contents",
                    req.reservation_id
                )))
            };
        }
        let Some(item) = stock.items.get_mut(&req.sku) else {
            return Err(Status::not_found(format!("no item with sku {:?}", req.sku)));
        };
        if item.available < req.quantity {
            return Err(Status::failed_precondition(format!(
                "only {} of {} available",
                item.available, req.sku
            )));
        }
        item.available -= req.quantity;
        let item = item.clone();
        let reservation = Reservation {
            reservation_id: req.reservation_id.clone(),
            sku: req.sku,
            quantity: req.quantity,
            remaining: item.available,
        };
        stock
            .reservations
            .insert(req.reservation_id, reservation.clone());
        drop(stock);
        self.publish(&item);
        Ok(Response::new(reservation))
    }

    async fn release(
        &self,
        request: Request<ReleaseRequest>,
    ) -> Result<Response<ReleaseResponse>, Status> {
        self.enter(&request).await?;
        let id = request.into_inner().reservation_id;
        let mut stock = self.stock.lock().unwrap();
        let Some(reservation) = stock.reservations.remove(&id) else {
            return Ok(Response::new(ReleaseResponse { released: false })); // already released: fine
        };
        let item = stock
            .items
            .get_mut(&reservation.sku)
            .expect("reserved items exist");
        item.available += reservation.quantity;
        let item = item.clone();
        drop(stock);
        self.publish(&item);
        Ok(Response::new(ReleaseResponse { released: true }))
    }

    type WatchStockStream = Pin<Box<dyn Stream<Item = Result<StockEvent, Status>> + Send>>;

    async fn watch_stock(
        &self,
        request: Request<WatchStockRequest>,
    ) -> Result<Response<Self::WatchStockStream>, Status> {
        self.enter(&request).await?;
        let sku = request.into_inner().sku;
        // Subscribe *before* reading the current level, so no change slips
        // between the two.
        let updates = BroadcastStream::new(self.events.subscribe());
        let current = {
            let stock = self.stock.lock().unwrap();
            let item = stock
                .items
                .get(&sku)
                .ok_or_else(|| Status::not_found(format!("no item with sku {sku:?}")))?;
            StockEvent {
                sku: sku.clone(),
                available: item.available,
            }
        };
        let changes = updates.filter_map(move |event| match event {
            Ok(e) if e.sku == sku => Some(Ok(e)),
            Ok(_) => None,
            // A slow watcher skipped some events; the next one has the current level anyway.
            Err(_lagged) => None,
        });
        Ok(Response::new(Box::pin(
            tokio_stream::once(Ok(current)).chain(changes),
        )))
    }
}
