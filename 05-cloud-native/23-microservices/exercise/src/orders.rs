//! Exercise 7: the Orders service -- a saga across services.
//!
//! Placing an order touches two services that each own their data:
//! Inventory (stock) and Payments (money). There's no transaction spanning
//! both. A **saga** is a sequence of local steps, each with a
//! **compensation** that undoes it if a later step fails:
//!
//! ```text
//!   reserve stock  ──ok──>  charge payment  ──ok──>  confirmed, publish OrderPlaced
//!                                │
//!                             declined
//!                                v
//!                   release stock (compensation)  ──>  rejected
//! ```
//!
//! Every step is idempotent (keyed by the order id), so retrying a failed
//! saga -- or the client retrying the whole request -- is safe.

use crate::breaker::{CircuitBreaker, counts_as_failure};
use crate::context::CallContext;
use crate::events::Bus;
use crate::pb::inventory_client::InventoryClient;
use crate::pb::orders_server::Orders;
use crate::pb::{
    GetItemRequest, Order, OrderState, PlaceOrderRequest, ReleaseRequest, ReserveRequest,
};
use crate::retry::{RetryPolicy, retry};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tonic::transport::Channel;
use tonic::{Code, Request, Response, Status};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentError {
    Declined(String),
}

pub trait Payments: Send + Sync {
    /// Idempotent per order id.
    fn charge(&self, order_id: &str, customer: &str, cents: u64) -> Result<(), PaymentError>;
}

/// Declines the customers it's told to; records successful charges.
#[derive(Default)]
pub struct FakePayments {
    declined: HashSet<String>,
    charges: Mutex<HashMap<String, u64>>,
}

impl FakePayments {
    pub fn declining(customers: &[&str]) -> FakePayments {
        FakePayments {
            declined: customers.iter().map(|c| c.to_string()).collect(),
            charges: Mutex::default(),
        }
    }

    pub fn charged(&self, order_id: &str) -> Option<u64> {
        self.charges.lock().unwrap().get(order_id).copied()
    }
}

impl Payments for FakePayments {
    fn charge(&self, order_id: &str, customer: &str, cents: u64) -> Result<(), PaymentError> {
        if self.declined.contains(customer) {
            return Err(PaymentError::Declined("card declined".into()));
        }
        self.charges
            .lock()
            .unwrap()
            .entry(order_id.to_string())
            .or_insert(cents);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaStep {
    Reserved,
    Charged,
    /// The compensation for Reserved.
    Released,
    Confirmed,
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderEvent {
    Placed {
        order_id: String,
        customer: String,
        sku: String,
        quantity: u32,
        total_cents: u64,
    },
    Rejected {
        order_id: String,
        reason: String,
    },
}

#[derive(Clone)]
pub struct OrdersService {
    inventory: InventoryClient<Channel>,
    payments: Arc<dyn Payments>,
    pub bus: Arc<Bus<OrderEvent>>,
    orders: Arc<Mutex<HashMap<String, Order>>>,
    sagas: Arc<Mutex<HashMap<String, Vec<SagaStep>>>>,
    pub breaker: Arc<Mutex<CircuitBreaker>>,
    retry: RetryPolicy,
}

impl OrdersService {
    pub fn new(
        inventory: InventoryClient<Channel>,
        payments: Arc<dyn Payments>,
        bus: Arc<Bus<OrderEvent>>,
        breaker: CircuitBreaker,
        retry: RetryPolicy,
    ) -> OrdersService {
        OrdersService {
            inventory,
            payments,
            bus,
            orders: Arc::default(),
            sagas: Arc::default(),
            breaker: Arc::new(Mutex::new(breaker)),
            retry,
        }
    }

    pub fn saga_log(&self, order_id: &str) -> Vec<SagaStep> {
        self.sagas
            .lock()
            .unwrap()
            .get(order_id)
            .cloned()
            .unwrap_or_default()
    }

    fn log(&self, order_id: &str, step: SagaStep) {
        self.sagas
            .lock()
            .unwrap()
            .entry(order_id.to_string())
            .or_default()
            .push(step);
    }

    /// An Inventory call through the circuit breaker, with retries.
    async fn call_inventory<T, F, Fut>(&self, mut op: F) -> Result<T, Status>
    where
        F: FnMut(InventoryClient<Channel>) -> Fut,
        Fut: Future<Output = Result<Response<T>, Status>>,
    {
        todo!("Exercise 7")
    }

    fn finish(&self, order: Order, event: OrderEvent, step: SagaStep) -> Order {
        todo!("Exercise 7")
    }

    fn reject(&self, order_id: &str, reason: &str) -> Order {
        todo!("Exercise 7")
    }
}

#[tonic::async_trait]
impl Orders for OrdersService {
    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> Result<Response<Order>, Status> {
        todo!("Exercise 7")
    }
}
