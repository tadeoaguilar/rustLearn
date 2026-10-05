// Reference solution for 23-microservices.
//
//     cargo run -p m23-microservices-solution -- demo    # both services on local ports, a client exercising them
//     cargo run -p m23-microservices-solution -- serve   # Inventory on :50051, Orders on :50052 (try grpcurl)

use m23_microservices_solution::balancer::Balancer;
use m23_microservices_solution::breaker::CircuitBreaker;
use m23_microservices_solution::events::{Bus, Idempotent};
use m23_microservices_solution::inventory::InventoryService;
use m23_microservices_solution::orders::{FakePayments, OrdersService};
use m23_microservices_solution::pb::inventory_client::InventoryClient;
use m23_microservices_solution::pb::inventory_server::InventoryServer;
use m23_microservices_solution::pb::orders_client::OrdersClient;
use m23_microservices_solution::pb::orders_server::OrdersServer;
use m23_microservices_solution::pb::{OrderState, PlaceOrderRequest, WatchStockRequest};
use m23_microservices_solution::retry::RetryPolicy;
use m23_microservices_solution::server::{lazy_channel, spawn_inventory, spawn_orders};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;

const CATALOG: &[(&str, &str, u32, u64)] = &[
    ("BOOK-1", "The Rust Book", 5, 3_999),
    ("MUG-7", "Ferris mug", 2, 1_250),
];

fn orders_service(inventory_url: &str, payments: Arc<FakePayments>) -> OrdersService {
    OrdersService::new(
        InventoryClient::new(lazy_channel(inventory_url)),
        payments,
        Arc::new(Bus::new(Duration::from_secs(30), 5)),
        CircuitBreaker::new(5, Duration::from_secs(5)),
        RetryPolicy {
            max_attempts: 4,
            base: Duration::from_millis(20),
            max_delay: Duration::from_millis(200),
        },
    )
}

fn order(id: &str, customer: &str, sku: &str, quantity: u32) -> PlaceOrderRequest {
    PlaceOrderRequest {
        order_id: id.into(),
        customer: customer.into(),
        sku: sku.into(),
        quantity,
    }
}

async fn demo() {
    let inventory = InventoryService::with_items(CATALOG);
    let inv = spawn_inventory(inventory.clone()).await;
    let payments = Arc::new(FakePayments::declining(&["mallory"]));
    let orders = orders_service(&inv.url, payments.clone());
    orders.bus.subscribe("emails");
    let ord = spawn_orders(orders.clone()).await;
    let mut client = OrdersClient::new(lazy_channel(&ord.url));
    println!("Inventory on {}, Orders on {}\n", inv.url, ord.url);

    // Watch BOOK-1's stock while ordering.
    let mut watch = InventoryClient::new(lazy_channel(&inv.url))
        .watch_stock(WatchStockRequest {
            sku: "BOOK-1".into(),
        })
        .await
        .unwrap()
        .into_inner();
    println!("watch BOOK-1: {:?}", watch.next().await.unwrap().unwrap());

    let show = |o: &m23_microservices_solution::pb::Order| {
        format!(
            "{} {:?} {} cents {}",
            o.order_id,
            OrderState::try_from(o.state).unwrap(),
            o.total_cents,
            o.reason
        )
    };
    let o1 = client
        .place_order(order("o-1", "alice", "BOOK-1", 2))
        .await
        .unwrap()
        .into_inner();
    println!("place o-1              -> {}", show(&o1));
    println!("watch BOOK-1: {:?}", watch.next().await.unwrap().unwrap());
    let again = client
        .place_order(order("o-1", "alice", "BOOK-1", 2))
        .await
        .unwrap()
        .into_inner();
    println!(
        "place o-1 again        -> {} (stock still {:?})",
        show(&again),
        inventory.available("BOOK-1")
    );
    let o2 = client
        .place_order(order("o-2", "mallory", "BOOK-1", 1))
        .await
        .unwrap()
        .into_inner();
    println!(
        "place o-2 (declined)   -> {}; saga {:?}; stock {:?}",
        show(&o2),
        orders.saga_log("o-2"),
        inventory.available("BOOK-1")
    );
    let o3 = client
        .place_order(order("o-3", "bob", "MUG-7", 5))
        .await
        .unwrap()
        .into_inner();
    println!("place o-3 (too many)   -> {}", show(&o3));
    println!(
        "place o-4 (bad sku)    -> {:?}",
        client
            .place_order(order("o-4", "bob", "NOPE", 1))
            .await
            .unwrap_err()
            .message()
    );

    inventory.fail_next(2);
    let calls = inventory.calls();
    let o5 = client
        .place_order(order("o-5", "bob", "MUG-7", 1))
        .await
        .unwrap()
        .into_inner();
    println!(
        "place o-5 (2 failures) -> {} after {} inventory calls",
        show(&o5),
        inventory.calls() - calls
    );

    let mut req = tonic::Request::new(order("o-6", "carol", "BOOK-1", 1));
    req.set_timeout(Duration::from_millis(100));
    inventory.set_delay(Duration::from_millis(300));
    let t = Instant::now();
    // tonic's server reports an expired grpc-timeout as CANCELLED.
    println!(
        "place o-6 (100ms deadline, slow inventory) -> {:?} in {:?}; inventory saw {:?}",
        client.place_order(req).await.unwrap_err().code(),
        t.elapsed(),
        inventory.last_seen()
    );
    inventory.set_delay(Duration::ZERO);

    inventory.fail_next(100);
    for i in 0..3 {
        let r = client
            .place_order(order(&format!("x-{i}"), "dave", "BOOK-1", 1))
            .await;
        println!(
            "inventory down, order x-{i} -> {:?} ({:?})",
            r.unwrap_err().message(),
            orders.breaker.lock().unwrap().state(Instant::now())
        );
    }
    inventory.fail_next(0);

    // Events: at-least-once, idempotent consumer.
    let mut sent = Idempotent::new();
    let start = Instant::now();
    while let Some(d) = orders.bus.poll("emails", start) {
        let handled = sent.handle(d.id, || println!("  email for {:?}", d.event));
        // Simulate a crash before the first ack of o-1's event: it comes back.
        if d.id == 1 && d.attempt == 1 {
            orders.bus.nack("emails", d.id, start);
        } else {
            orders.bus.ack("emails", d.id);
        }
        if !handled {
            println!("  (duplicate delivery of event {} ignored)", d.id);
        }
    }

    // Bonus: two Inventory instances, one goes away.
    let mut a = spawn_inventory(InventoryService::with_items(CATALOG)).await;
    let b = spawn_inventory(InventoryService::with_items(CATALOG)).await;
    let lb = Balancer::new(&[a.url.clone(), b.url.clone()], Duration::from_secs(10));
    for _ in 0..4 {
        println!(
            "balanced GetItem -> {}",
            lb.get_item("MUG-7").await.unwrap().0
        );
    }
    a.stop();
    tokio::time::sleep(Duration::from_millis(50)).await;
    for _ in 0..3 {
        println!(
            "after stopping {} -> {:?}",
            a.url,
            lb.get_item("MUG-7").await.map(|r| r.0)
        );
    }
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("demo") | Some("all") => demo().await,
        Some("serve") => {
            let inventory = InventoryService::with_items(CATALOG);
            let orders = orders_service(
                "http://127.0.0.1:50051",
                Arc::new(FakePayments::declining(&["mallory"])),
            );
            println!("Inventory on 127.0.0.1:50051, Orders on 127.0.0.1:50052 -- Ctrl-C to stop");
            let inv = tonic::transport::Server::builder()
                .add_service(InventoryServer::new(inventory))
                .serve("127.0.0.1:50051".parse().unwrap());
            let ord = tonic::transport::Server::builder()
                .add_service(OrdersServer::new(orders))
                .serve("127.0.0.1:50052".parse().unwrap());
            let (a, b) = tokio::join!(inv, ord);
            a.unwrap();
            b.unwrap();
        }
        _ => {
            println!("23-microservices -- reference solution\n");
            println!(
                "  cargo run -p m23-microservices-solution -- demo    both services and a client"
            );
            println!(
                "  cargo run -p m23-microservices-solution -- serve   Inventory :50051, Orders :50052"
            );
        }
    }
}
