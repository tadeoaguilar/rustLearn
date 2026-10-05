# Exercises: Microservices

Two services that own their own data and talk over gRPC — **Inventory**
(stock) and **Orders** (places orders by calling Inventory and a payment
provider) — and everything that makes such a system hold up when the network
and the other services don't: deadlines, retries, circuit breakers, events
and sagas.

**Setup**: `proto/shop.proto` is the contract; `build.rs` generates Rust from
it with [tonic](https://docs.rs/tonic) and `protox` (pure Rust — no `protoc`
to install). `server.rs` (provided) starts services on random local ports.
The generated code is in `pb`: `pb::inventory_server::Inventory` is the trait
you implement, `pb::inventory_client::InventoryClient` the client.

---

## Exercise 1: A gRPC Service

**Difficulty**: Medium
**Time**: 50 minutes

**Learning Objectives**:
- Implement a tonic service
- Choose the right status code
- Make an operation idempotent

Implement `Inventory` for `InventoryService` (`inventory.rs`; the state and
test hooks are provided):

| RPC | Errors |
|---|---|
| `GetItem` | `NotFound` |
| `Reserve` | empty id or quantity 0: `InvalidArgument`; unknown SKU: `NotFound`; not enough stock: `FailedPrecondition`; same id, different contents: `AlreadyExists` |
| `Release` | none: releasing an unknown or released reservation returns `released: false` |

`Reserve` with a reservation id that already succeeded returns the original
reservation **without reserving again**. Publish a `StockEvent` on every change.

**Question**: why does `Reserve` take a client-chosen `reservation_id`?

---

## Exercise 2: Server Streaming

**Difficulty**: Medium
**Time**: 30 minutes

`WatchStock(sku)` returns a stream: the current level first, then every change
for that SKU (`NotFound` for unknown SKUs).

```rust
type WatchStockStream = Pin<Box<dyn Stream<Item = Result<StockEvent, Status>> + Send>>;
```

**Hint**: `BroadcastStream` turns a `broadcast::Receiver` into a `Stream`;
`tokio_stream::once(current).chain(changes)`. Subscribe *before* reading the
current level — why?

---

## Exercise 3: Request IDs and Deadlines

**Difficulty**: Medium
**Time**: 40 minutes

`context.rs`:

```rust
pub fn parse_grpc_timeout(value: &str) -> Option<Duration>;   // "250m", "2S", "1H"... up to 8 digits
impl CallContext {
    pub fn from_request<T>(request: &Request<T>) -> CallContext;  // x-request-id (or a new one), deadline
    pub fn remaining(&self) -> Option<Duration>;
    pub fn outgoing<T>(&self, message: T) -> Result<Request<T>, Status>;  // id + remaining time; fail if expired
}
```

Units: `H` hours, `M` minutes, `S` seconds, `m` millis, `u` micros, `n` nanos.

**Question**: the client allows 2 s. Orders spends 1.5 s, then calls Inventory
with no deadline, which takes 3 s. What goes wrong?

---

## Exercise 4: Retries With Backoff and Jitter

**Difficulty**: Medium
**Time**: 40 minutes

`retry.rs`:

```rust
pub fn is_retryable(code: Code) -> bool;   // Unavailable, ResourceExhausted, Aborted
pub fn backoff(policy: &RetryPolicy, attempt: u32, rng: &mut impl Rng) -> Duration;  // full jitter
pub async fn retry<T, F, Fut>(policy: &RetryPolicy, op: F) -> Result<T, Status>;
```

"Full jitter" picks uniformly between zero and `min(max_delay, base · 2^attempt)`.
`max_attempts` counts the first try.

---

## Exercise 5: A Circuit Breaker

**Difficulty**: Hard
**Time**: 50 minutes

`breaker.rs` — Closed → (N consecutive failures) → Open → (cool-down) →
HalfOpen → one probe → Closed or Open. Time is passed in as `now`.

```rust
pub fn try_acquire(&mut self, now: Instant) -> bool;
pub fn on_success(&mut self);
pub fn on_failure(&mut self, now: Instant);
pub fn counts_as_failure(code: Code) -> bool;   // Unavailable yes, NotFound no
```

Watch out: the caller of a half-open probe may give up (a deadline) and never
report back. The breaker must not stay half-open forever.

---

## Exercise 6: Events, At Least Once

**Difficulty**: Hard
**Time**: 50 minutes

`events.rs` — an in-memory broker with the semantics of SQS/RabbitMQ:

- `subscribe(group)`: the group receives events published from then on;
  every group gets every event
- `poll(group, now)`: delivers the next event; it's then invisible for the
  visibility timeout
- `ack` removes it; `nack` (or the timeout) makes it available again
- after `max_deliveries` deliveries without an ack: the dead-letter queue

Then `Idempotent::handle(id, handler)` runs each event id's handler once.

---

## Exercise 7: The Orders Saga

**Difficulty**: Very Hard
**Time**: 90 minutes

Implement `Orders::place_order` and `OrdersService::call_inventory`:

```
validate (InvalidArgument) -> already placed? return it
GetItem (price; NotFound -> InvalidArgument "unknown sku")
Reserve (reservation id = order id)   FailedPrecondition -> Rejected "out of stock"
charge payment                        Declined -> Release (compensation) -> Rejected
confirmed -> publish OrderPlaced
```

Every Inventory call goes through the circuit breaker and `retry`, carries the
request id and the remaining deadline, and is logged in the saga log. Rejected
orders publish `OrderEvent::Rejected`.

**Question**: what if the compensation (Release) itself fails?

---

## Bonus Challenge: Client-Side Load Balancing

**Difficulty**: Medium
**Time**: 40 minutes

`balancer.rs`: round-robin over several Inventory instances, skipping those
marked down; on `Unavailable`, mark the instance down for a cool-down and try
the next. A `NotFound` is a real answer, not a reason to fail over.

---

## Check Your Understanding

- [ ] Define a service in protobuf and evolve it safely
- [ ] Pick gRPC status codes that tell the caller what to do
- [ ] Make operations idempotent with client-chosen keys
- [ ] Propagate deadlines and request ids
- [ ] Retry only what's safe, with backoff and jitter
- [ ] Explain the circuit breaker's three states
- [ ] Explain at-least-once delivery and idempotent consumers
- [ ] Design a saga with compensations

---

## Additional Resources

- [tonic](https://docs.rs/tonic) and its [examples](https://github.com/hyperium/tonic/tree/master/examples)
- [gRPC status codes](https://grpc.io/docs/guides/status-codes/) and [deadlines](https://grpc.io/docs/guides/deadlines/)
- [Exponential Backoff And Jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/) — AWS Builders' Library
- [Martin Fowler: CircuitBreaker](https://martinfowler.com/bliki/CircuitBreaker.html)
- [microservices.io: Saga](https://microservices.io/patterns/data/saga.html), [Transactional outbox](https://microservices.io/patterns/data/transactional-outbox.html)
- [Protocol Buffers: updating a message type](https://protobuf.dev/programming-guides/proto3/#updating)
