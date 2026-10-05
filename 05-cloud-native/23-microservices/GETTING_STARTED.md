# Getting Started with 23 · Microservices

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m23-microservices-solution -- demo
```

starts Inventory and Orders on local ports, then places orders through every
path: success, an idempotent retry, a declined payment (compensated), out of
stock, an unknown SKU, transient failures (retried), a deadline, an outage
(the breaker opens), event delivery with a duplicate, and load balancing with
failover.

## What Is Already Here

```
23-microservices/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/                # ← YOUR WORKSPACE (package m23-microservices)
│   ├── proto/shop.proto     #   the contract
│   ├── build.rs             #   code generation (protox + tonic-prost-build)
│   ├── src/inventory.rs     #   Ex 1, 2 (state and test hooks provided)
│   ├── src/context.rs       #   Ex 3
│   ├── src/retry.rs         #   Ex 4
│   ├── src/breaker.rs       #   Ex 5
│   ├── src/events.rs        #   Ex 6
│   ├── src/orders.rs        #   Ex 7 (FakePayments provided)
│   ├── src/balancer.rs      #   bonus
│   └── src/server.rs        #   provided: spawn servers on local ports
├── solution/                # ← REFERENCE (m23-microservices-solution) + ANSWERS.md
└── tests/                   # ← 17 tests (m23-microservices-tests)
```

To see the generated code: `cargo build -p m23-microservices`, then open
`target/debug/build/m23-microservices-*/out/shop.v1.rs`.

## The Commands You Need

```bash
cargo run  -p m23-microservices -- demo
cargo test -p m23-microservices-tests --features mine
cargo test -p m23-microservices-tests --features mine ex5_
cargo test -p m23-microservices-tests                    # the solution: always green
```

## Talk to the Services by Hand (Optional)

With [grpcurl](https://github.com/fullstorydev/grpcurl) (`brew install grpcurl`)
and `cargo run -p m23-microservices-solution -- serve` running:

```bash
P=05-cloud-native/23-microservices/solution/proto
grpcurl -plaintext -import-path $P -proto shop.proto -d '{"sku":"BOOK-1"}' 127.0.0.1:50051 shop.v1.Inventory/GetItem
grpcurl -plaintext -import-path $P -proto shop.proto -d '{"sku":"BOOK-1"}' 127.0.0.1:50051 shop.v1.Inventory/WatchStock   # keeps streaming
grpcurl -plaintext -import-path $P -proto shop.proto -H 'x-request-id: demo-1' -max-time 2 \
    -d '{"order_id":"o-1","customer":"alice","sku":"BOOK-1","quantity":2}' 127.0.0.1:50052 shop.v1.Orders/PlaceOrder
```

## If You Get Stuck

1. **`async fn` in the impl doesn't match the trait** — put `#[tonic::async_trait]` on the `impl` block.
2. **"future cannot be sent between threads safely"** — a `MutexGuard` or `ThreadRng` lives across an `.await`; drop it first (end its scope).
3. **WatchStock test misses an update** — subscribe to the broadcast channel before reading the current level.
4. **Reserve counts twice on retry** — look the reservation id up *before* touching the stock.
5. **Breaker tests** — all transitions depend on `now` only; no sleeping needed.
6. **The deadline test hangs** — `CallContext::outgoing` must call `request.set_timeout(remaining)`.
7. Compare with `solution/src/` — same file and function names.
