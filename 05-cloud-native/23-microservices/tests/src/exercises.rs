use crate::sut::*;
use breaker::{CircuitBreaker, State};
use context::CallContext;
use events::{Bus, Idempotent};
use inventory::InventoryService;
use orders::{FakePayments, OrderEvent, OrdersService, SagaStep};
use pb::inventory_client::InventoryClient;
use pb::orders_client::OrdersClient;
use pb::*;
use rand::SeedableRng;
use retry::RetryPolicy;
use server::{Running, lazy_channel, spawn_inventory, spawn_orders};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;
use tonic::Code;

const CATALOG: &[(&str, &str, u32, u64)] = &[
    ("BOOK-1", "The Rust Book", 5, 3_999),
    ("MUG-7", "Ferris mug", 2, 1_250),
];

async fn inventory() -> (
    InventoryService,
    Running,
    InventoryClient<tonic::transport::Channel>,
) {
    let svc = InventoryService::with_items(CATALOG);
    let running = spawn_inventory(svc.clone()).await;
    let client = InventoryClient::new(lazy_channel(&running.url));
    (svc, running, client)
}

fn reserve(id: &str, sku: &str, quantity: u32) -> ReserveRequest {
    ReserveRequest {
        reservation_id: id.into(),
        sku: sku.into(),
        quantity,
    }
}

// ---- Exercise 1: a gRPC service ----------------------------------------------------------------

#[tokio::test]
async fn ex1_get_item_and_status_codes() {
    let (_svc, _r, mut c) = inventory().await;
    let item = c
        .get_item(GetItemRequest {
            sku: "BOOK-1".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        (item.name.as_str(), item.available, item.price_cents),
        ("The Rust Book", 5, 3999)
    );
    assert_eq!(
        c.get_item(GetItemRequest { sku: "NOPE".into() })
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        c.reserve(reserve("", "BOOK-1", 1))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        c.reserve(reserve("r", "BOOK-1", 0))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        c.reserve(reserve("r", "NOPE", 1)).await.unwrap_err().code(),
        Code::NotFound
    );
    assert_eq!(
        c.reserve(reserve("r", "MUG-7", 3))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
}

#[tokio::test]
async fn ex1_reserve_is_idempotent_and_release_restores() {
    let (svc, _r, mut c) = inventory().await;
    let first = c
        .reserve(reserve("r-1", "BOOK-1", 2))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.remaining, 3);
    let again = c
        .reserve(reserve("r-1", "BOOK-1", 2))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(again, first, "a retried reservation returns the original");
    assert_eq!(
        svc.available("BOOK-1"),
        Some(3),
        "and doesn't reserve twice"
    );
    assert_eq!(
        c.reserve(reserve("r-1", "BOOK-1", 1))
            .await
            .unwrap_err()
            .code(),
        Code::AlreadyExists
    );

    assert!(
        c.release(ReleaseRequest {
            reservation_id: "r-1".into()
        })
        .await
        .unwrap()
        .into_inner()
        .released
    );
    assert!(
        !c.release(ReleaseRequest {
            reservation_id: "r-1".into()
        })
        .await
        .unwrap()
        .into_inner()
        .released,
        "second release is a no-op"
    );
    assert_eq!(svc.available("BOOK-1"), Some(5));
}

// ---- Exercise 2: server streaming ----------------------------------------------------------------

async fn next(watch: &mut tonic::Streaming<StockEvent>) -> u32 {
    tokio::time::timeout(Duration::from_secs(2), watch.next())
        .await
        .expect("an event")
        .unwrap()
        .unwrap()
        .available
}

#[tokio::test]
async fn ex2_watch_stock_streams_current_then_changes() {
    let (_svc, _r, mut c) = inventory().await;
    let mut watch = c
        .watch_stock(WatchStockRequest {
            sku: "BOOK-1".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(next(&mut watch).await, 5, "the current level first");
    c.reserve(reserve("a", "MUG-7", 1)).await.unwrap(); // another SKU: filtered out
    c.reserve(reserve("b", "BOOK-1", 2)).await.unwrap();
    assert_eq!(next(&mut watch).await, 3);
    c.release(ReleaseRequest {
        reservation_id: "b".into(),
    })
    .await
    .unwrap();
    assert_eq!(next(&mut watch).await, 5);
    assert_eq!(
        c.watch_stock(WatchStockRequest { sku: "NOPE".into() })
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
}

// ---- Exercise 3: context propagation ----------------------------------------------------------------

#[test]
fn ex3_parse_grpc_timeout() {
    use context::parse_grpc_timeout as p;
    assert_eq!(p("250m"), Some(Duration::from_millis(250)));
    assert_eq!(p("2S"), Some(Duration::from_secs(2)));
    assert_eq!(p("1H"), Some(Duration::from_secs(3600)));
    assert_eq!(p("3M"), Some(Duration::from_secs(180)));
    assert_eq!(p("10u"), Some(Duration::from_micros(10)));
    assert_eq!(p("99999999n"), Some(Duration::from_nanos(99_999_999)));
    for bad in ["", "m", "123456789m", "10x", "-5m", "1.5S"] {
        assert_eq!(p(bad), None, "{bad:?}");
    }
}

#[test]
fn ex3_call_context() {
    let mut incoming = tonic::Request::new(());
    incoming
        .metadata_mut()
        .insert("x-request-id", "abc-123".parse().unwrap());
    incoming
        .metadata_mut()
        .insert("grpc-timeout", "500m".parse().unwrap());
    let ctx = CallContext::from_request(&incoming);
    assert_eq!(ctx.request_id, "abc-123");
    let left = ctx.remaining().unwrap();
    assert!(left <= Duration::from_millis(500) && left > Duration::from_millis(400));

    let out = ctx.outgoing(GetItemRequest::default()).unwrap();
    assert_eq!(out.metadata().get("x-request-id").unwrap(), "abc-123");
    assert!(
        out.metadata().get("grpc-timeout").is_some(),
        "the remaining time travels on"
    );

    let fresh = CallContext::from_request(&tonic::Request::new(()));
    assert!(!fresh.request_id.is_empty() && fresh.deadline.is_none());
    assert_ne!(
        fresh.request_id,
        CallContext::from_request(&tonic::Request::new(())).request_id,
        "new ids are unique"
    );

    let expired = CallContext {
        request_id: "x".into(),
        deadline: Some(Instant::now() - Duration::from_millis(1)),
    };
    assert_eq!(
        expired.outgoing(()).unwrap_err().code(),
        Code::DeadlineExceeded
    );
}

// ---- Exercise 4: retries ---------------------------------------------------------------------------

#[test]
fn ex4_backoff_has_jitter_and_a_cap() {
    let policy = RetryPolicy {
        max_attempts: 10,
        base: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(7);
    for attempt in 0..10 {
        let cap = Duration::from_millis(100 * 2u64.pow(attempt)).min(Duration::from_secs(1));
        for _ in 0..50 {
            assert!(retry::backoff(&policy, attempt, &mut rng) <= cap);
        }
    }
    let delays: Vec<Duration> = (0..20)
        .map(|_| retry::backoff(&policy, 3, &mut rng))
        .collect();
    assert!(
        delays.iter().any(|d| *d != delays[0]),
        "jitter: not every delay is the same"
    );
    assert!(
        retry::backoff(&policy, 1000, &mut rng) <= Duration::from_secs(1),
        "no overflow"
    );
    assert!(retry::is_retryable(Code::Unavailable));
    assert!(!retry::is_retryable(Code::InvalidArgument) && !retry::is_retryable(Code::NotFound));
}

#[tokio::test]
async fn ex4_retry_only_retryable_errors() {
    let policy = RetryPolicy {
        max_attempts: 4,
        base: Duration::from_millis(1),
        max_delay: Duration::from_millis(5),
    };
    let calls = AtomicU32::new(0);
    let ok = retry::retry(&policy, |attempt| {
        calls.fetch_add(1, Ordering::SeqCst);
        async move {
            if attempt < 2 {
                Err(tonic::Status::unavailable("blip"))
            } else {
                Ok(attempt)
            }
        }
    })
    .await;
    assert_eq!(ok.unwrap(), 2);
    assert_eq!(calls.load(Ordering::SeqCst), 3);

    calls.store(0, Ordering::SeqCst);
    let err = retry::retry(&policy, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        async { Err::<(), _>(tonic::Status::not_found("nope")) }
    })
    .await;
    assert_eq!(
        (err.unwrap_err().code(), calls.load(Ordering::SeqCst)),
        (Code::NotFound, 1),
        "not retried"
    );

    calls.store(0, Ordering::SeqCst);
    let err = retry::retry(&policy, |_| {
        calls.fetch_add(1, Ordering::SeqCst);
        async { Err::<(), _>(tonic::Status::unavailable("down")) }
    })
    .await;
    assert_eq!(
        (err.unwrap_err().code(), calls.load(Ordering::SeqCst)),
        (Code::Unavailable, 4),
        "gave up after max_attempts"
    );
}

// ---- Exercise 5: circuit breaker ----------------------------------------------------------------

#[test]
fn ex5_breaker_state_machine() {
    let t0 = Instant::now();
    let s = |n: u64| t0 + Duration::from_secs(n);
    let mut b = CircuitBreaker::new(3, Duration::from_secs(10));
    for _ in 0..2 {
        assert!(b.try_acquire(t0));
        b.on_failure(t0);
    }
    assert_eq!(b.state(t0), State::Closed);
    b.on_success();
    for _ in 0..2 {
        b.on_failure(t0);
    }
    assert_eq!(b.state(t0), State::Closed, "a success resets the count");
    b.on_failure(t0);
    assert_eq!(b.state(s(1)), State::Open);
    assert!(!b.try_acquire(s(5)), "open: fail fast");

    assert_eq!(b.state(s(10)), State::HalfOpen);
    assert!(b.try_acquire(s(10)), "one probe");
    assert!(!b.try_acquire(s(11)), "only one");
    b.on_failure(s(11));
    assert_eq!(
        b.state(s(15)),
        State::Open,
        "failed probe: open again for a full cool-down"
    );
    assert!(b.try_acquire(s(21)));
    b.on_success();
    assert_eq!(b.state(s(21)), State::Closed);
}

#[test]
fn ex5_a_lost_probe_does_not_wedge_the_breaker() {
    let t0 = Instant::now();
    let mut b = CircuitBreaker::new(1, Duration::from_secs(10));
    b.on_failure(t0);
    assert!(
        b.try_acquire(t0 + Duration::from_secs(10)),
        "probe; its caller then gives up and never reports"
    );
    assert!(!b.try_acquire(t0 + Duration::from_secs(12)));
    assert!(
        b.try_acquire(t0 + Duration::from_secs(21)),
        "after another cool-down, a new probe"
    );
    assert!(
        breaker::counts_as_failure(Code::Unavailable)
            && !breaker::counts_as_failure(Code::NotFound)
    );
}

// ---- Exercise 6: events --------------------------------------------------------------------------

#[test]
fn ex6_groups_ack_redelivery_dead_letters() {
    let t0 = Instant::now();
    let bus: Bus<&str> = Bus::new(Duration::from_secs(30), 3);
    bus.publish("before anyone subscribed");
    bus.subscribe("email");
    bus.subscribe("analytics");
    let a = bus.publish("order 1");
    let b = bus.publish("order 2");

    let d = bus.poll("email", t0).unwrap();
    assert_eq!((d.id, d.event, d.attempt), (a, "order 1", 1));
    bus.ack("email", a);
    assert_eq!(
        bus.poll("analytics", t0).unwrap().id,
        a,
        "each group gets every event"
    );

    let d = bus.poll("email", t0).unwrap();
    assert_eq!(d.id, b);
    // Not acked: invisible until the timeout, then delivered again.
    assert!(bus.poll("email", t0 + Duration::from_secs(29)).is_none());
    let again = bus.poll("email", t0 + Duration::from_secs(30)).unwrap();
    assert_eq!((again.id, again.attempt), (b, 2));
    bus.nack("email", b, t0 + Duration::from_secs(31));
    assert_eq!(
        bus.poll("email", t0 + Duration::from_secs(31))
            .unwrap()
            .attempt,
        3
    );
    bus.nack("email", b, t0 + Duration::from_secs(32));
    assert!(
        bus.poll("email", t0 + Duration::from_secs(32)).is_none(),
        "3 deliveries: dead-lettered"
    );
    assert_eq!(bus.dead_letters("email"), vec![(b, "order 2")]);
    assert_eq!(bus.pending("email"), 0);
    assert!(bus.poll("nobody", t0).is_none());
}

#[test]
fn ex6_idempotent_consumer() {
    let mut consumer = Idempotent::new();
    let mut sent = Vec::new();
    for id in [1, 2, 1, 3, 2] {
        consumer.handle(id, || sent.push(id));
    }
    assert_eq!(sent, [1, 2, 3]);
}

// ---- Exercise 7: the Orders saga -------------------------------------------------------------------

struct Shop {
    inventory: InventoryService,
    _inv: Running,
    _ord: Running,
    orders: OrdersService,
    payments: Arc<FakePayments>,
    client: OrdersClient<tonic::transport::Channel>,
}

async fn shop() -> Shop {
    let inventory = InventoryService::with_items(CATALOG);
    let inv = spawn_inventory(inventory.clone()).await;
    let payments = Arc::new(FakePayments::declining(&["mallory"]));
    let bus = Arc::new(Bus::new(Duration::from_secs(30), 5));
    bus.subscribe("test");
    let orders = OrdersService::new(
        InventoryClient::new(lazy_channel(&inv.url)),
        payments.clone(),
        bus,
        CircuitBreaker::new(5, Duration::from_secs(30)),
        RetryPolicy {
            max_attempts: 4,
            base: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        },
    );
    let ord = spawn_orders(orders.clone()).await;
    let client = OrdersClient::new(lazy_channel(&ord.url));
    Shop {
        inventory,
        _inv: inv,
        _ord: ord,
        orders,
        payments,
        client,
    }
}

fn order(id: &str, customer: &str, sku: &str, quantity: u32) -> PlaceOrderRequest {
    PlaceOrderRequest {
        order_id: id.into(),
        customer: customer.into(),
        sku: sku.into(),
        quantity,
    }
}

fn events(s: &Shop) -> Vec<OrderEvent> {
    let mut out = Vec::new();
    while let Some(d) = s.orders.bus.poll("test", Instant::now()) {
        s.orders.bus.ack("test", d.id);
        out.push(d.event);
    }
    out
}

#[tokio::test]
async fn ex7_happy_path_and_idempotent_retry() {
    let mut s = shop().await;
    let o = s
        .client
        .place_order(order("o-1", "alice", "BOOK-1", 2))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        (o.state, o.total_cents),
        (OrderState::Confirmed as i32, 7998)
    );
    assert_eq!(
        s.orders.saga_log("o-1"),
        [SagaStep::Reserved, SagaStep::Charged, SagaStep::Confirmed]
    );
    assert_eq!(s.payments.charged("o-1"), Some(7998));
    assert_eq!(s.inventory.available("BOOK-1"), Some(3));

    let again = s
        .client
        .place_order(order("o-1", "alice", "BOOK-1", 2))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(again, o);
    assert_eq!(
        s.inventory.available("BOOK-1"),
        Some(3),
        "a retried order doesn't reserve twice"
    );
    assert_eq!(
        events(&s),
        [OrderEvent::Placed {
            order_id: "o-1".into(),
            customer: "alice".into(),
            sku: "BOOK-1".into(),
            quantity: 2,
            total_cents: 7998
        }]
    );
}

#[tokio::test]
async fn ex7_payment_failure_compensates() {
    let mut s = shop().await;
    let o = s
        .client
        .place_order(order("o-2", "mallory", "BOOK-1", 1))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(o.state, OrderState::Rejected as i32);
    assert!(o.reason.contains("payment"), "{}", o.reason);
    assert_eq!(
        &s.orders.saga_log("o-2")[..2],
        [SagaStep::Reserved, SagaStep::Released]
    );
    assert_eq!(
        s.inventory.available("BOOK-1"),
        Some(5),
        "the reservation was released"
    );
    assert!(
        matches!(&events(&s)[..], [OrderEvent::Rejected { order_id, .. }] if order_id == "o-2")
    );
}

#[tokio::test]
async fn ex7_out_of_stock_bad_input_and_unknown_sku() {
    let mut s = shop().await;
    let o = s
        .client
        .place_order(order("o-3", "bob", "MUG-7", 5))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        (o.state, o.reason.as_str()),
        (OrderState::Rejected as i32, "out of stock")
    );
    assert_eq!(
        s.client
            .place_order(order("o-4", "bob", "NOPE", 1))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        s.client
            .place_order(order("", "bob", "MUG-7", 1))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        s.client
            .place_order(order("o-5", "bob", "MUG-7", 0))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
}

#[tokio::test]
async fn ex7_retries_through_blips_and_opens_the_breaker_on_outages() {
    let mut s = shop().await;
    s.inventory.fail_next(2);
    let o = s
        .client
        .place_order(order("o-6", "bob", "MUG-7", 1))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        o.state,
        OrderState::Confirmed as i32,
        "two UNAVAILABLEs are retried away"
    );

    s.inventory.fail_next(1000);
    for i in 0..3 {
        let err = s
            .client
            .place_order(order(&format!("x-{i}"), "bob", "MUG-7", 1))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::Unavailable);
    }
    assert_eq!(
        s.orders.breaker.lock().unwrap().state(Instant::now()),
        State::Open
    );
    let calls = s.inventory.calls();
    let _ = s.client.place_order(order("x-9", "bob", "MUG-7", 1)).await;
    assert_eq!(
        s.inventory.calls(),
        calls,
        "an open breaker doesn't call the failing service at all"
    );
}

#[tokio::test]
async fn ex7_deadline_and_request_id_propagate() {
    let mut s = shop().await;
    s.inventory.set_delay(Duration::from_millis(500));
    let mut req = tonic::Request::new(order("o-7", "carol", "BOOK-1", 1));
    req.set_timeout(Duration::from_millis(150));
    req.metadata_mut()
        .insert("x-request-id", "trace-me".parse().unwrap());
    let t = Instant::now();
    let err = s.client.place_order(req).await.unwrap_err();
    assert!(
        matches!(err.code(), Code::DeadlineExceeded | Code::Cancelled),
        "{err:?}"
    );
    assert!(
        t.elapsed() < Duration::from_millis(450),
        "the client didn't wait for the slow inventory"
    );
    let (request_id, timeout) = s.inventory.last_seen().expect("inventory was called");
    assert_eq!(request_id.as_deref(), Some("trace-me"));
    assert!(
        timeout.expect("a grpc-timeout was sent") <= Duration::from_millis(150),
        "inventory got the *remaining* time"
    );
}

// ---- Bonus: load balancing -------------------------------------------------------------------------

#[tokio::test]
async fn bonus_round_robin_and_failover() {
    let mut a = spawn_inventory(InventoryService::with_items(CATALOG)).await;
    let b = spawn_inventory(InventoryService::with_items(CATALOG)).await;
    let lb = balancer::Balancer::new(&[a.url.clone(), b.url.clone()], Duration::from_secs(60));
    let mut hits = Vec::new();
    for _ in 0..4 {
        hits.push(lb.get_item("MUG-7").await.unwrap().0);
    }
    assert_eq!(
        hits,
        [a.url.clone(), b.url.clone(), a.url.clone(), b.url.clone()]
    );

    a.stop();
    tokio::time::sleep(Duration::from_millis(100)).await;
    for _ in 0..4 {
        assert_eq!(
            lb.get_item("MUG-7").await.unwrap().0,
            b.url,
            "only the healthy instance"
        );
    }
    assert_eq!(
        lb.get_item("NOPE").await.unwrap_err().code(),
        Code::NotFound,
        "real errors aren't failover reasons"
    );
}
