# 23 · Microservices

## Overview

A microservice architecture splits a system into services that each own their
data and deploy independently. That buys team autonomy and independent
scaling — and costs you every guarantee a single process gave for free. Calls
can be slow, fail, or succeed without you hearing about it; there's no
transaction across services. This module builds two gRPC services with
[tonic](https://docs.rs/tonic) and the patterns that keep them correct anyway.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | A gRPC service | Protobuf contracts, status codes, idempotency keys |
| 2 | Streaming | Server-streaming RPCs from a broadcast channel |
| 3 | Context | Request ids and deadlines across hops |
| 4 | Retries | Retryable codes, exponential backoff, full jitter |
| 5 | Circuit breaker | Failing fast; half-open probes; not getting stuck |
| 6 | Events | At-least-once delivery, acks, redelivery, dead letters, idempotent consumers |
| 7 | Saga | A cross-service operation with compensation |
| Bonus | Load balancing | Client-side round robin with failover |

## Key Concepts

### The call chain

```
client ──PlaceOrder (deadline 2 s, x-request-id)──▶ Orders
                                                   ├──GetItem  (remaining 1.9 s, same id)──▶ Inventory
                                                   ├──Reserve  (remaining 1.8 s)──────────▶ Inventory
                                                   ├──charge ─────────────────────────────▶ Payments
                                                   └──publish OrderPlaced ──▶ event bus ──▶ consumers
```

### Resilience patterns, and where each goes

| Problem | Pattern | Here |
|---|---|---|
| a request hangs | deadline, propagated | `context.rs` |
| a blip | retry with backoff + jitter, idempotent operations | `retry.rs`, reservation ids |
| an outage | circuit breaker | `breaker.rs` |
| one instance down | load balancing with failover | `balancer.rs` |
| no distributed transaction | saga with compensations | `orders.rs` |
| a consumer crashes mid-event | at-least-once delivery + idempotent consumers | `events.rs` |

### Why gRPC

Protobuf contracts generate client and server code in many languages, binary
encoding is compact, HTTP/2 multiplexes calls and supports streaming, and
deadlines and status codes are part of the protocol. The cost: not
human-readable, and browsers need a proxy (gRPC-Web).

## Common Pitfalls

1. **Retrying non-idempotent calls** — a timed-out "charge card" may have succeeded
2. **Retries at every layer** — 3 retries × 3 layers = 27 calls to a struggling service
3. **No deadline** — work continues long after the user gave up
4. **Treating `NotFound` as a failure** in a breaker — a healthy service opens the circuit
5. **Publishing events outside the transaction** — the DB commit and the publish can diverge (use an outbox)
6. **Assuming exactly-once delivery** — every consumer will eventually see duplicates
7. **Sharing a database between services** — they're no longer independent

## Running This Module

```bash
cargo run  -p m23-microservices -- demo                  # your code
cargo test -p m23-microservices-tests --features mine    # test your code
cargo run  -p m23-microservices-solution -- demo         # both services, every pattern
cargo run  -p m23-microservices-solution -- serve        # Inventory :50051, Orders :50052
cargo test -p m23-microservices-tests                    # 17 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a gRPC service with Tonic, service-to-service
communication, circuit breakers and retries, an event-driven system).

- **The message broker is in-memory** (`events.rs`) with the semantics of a
  real one, so tests run without RabbitMQ or Kafka. `lapin` (RabbitMQ) and
  `rdkafka` follow the same publish / consume / ack shape.
- **Payments are a fake**, as an external provider would be in tests.
- **tonic reports an expired deadline as `CANCELLED`** on the server side
  rather than `DEADLINE_EXCEEDED`; the tests accept either.
- **Service discovery** (Consul, etcd) is reduced to a list of URLs in the
  bonus; in Kubernetes, a Service's DNS name or a headless Service fills that role.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[24 · Observability](../24-observability/)
