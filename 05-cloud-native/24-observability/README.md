# 24 · Observability

## Overview

Monitoring tells you *that* something is wrong; observability lets you find
out *why*, including for failures nobody predicted. In practice that means
emitting three kinds of telemetry — structured logs, metrics and traces — in
a form tools can query, and turning them into a few alerts that matter. Rust's
`tracing` crate covers logs and spans; `prometheus-client` covers metrics;
the W3C `traceparent` header ties services together.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Structured logs | `tracing` events and spans, JSON output |
| 2 | Filtering, redaction | Per-module levels; no personal data in logs |
| 3 | Metrics | RED metrics; histograms; cardinality |
| 4 | Trace context | The `traceparent` header |
| 5 | Distributed tracing | A `tracing` Layer that assembles traces across two services |
| 6 | SLOs | Error budgets and multiwindow burn-rate alerts |
| 7 | As code | Grafana dashboards and Prometheus rules generated from Rust |
| Bonus | Sampling | Consistent head-based sampling |

## Key Concepts

### Events and spans

```rust
#[instrument(fields(order_id = %order.id))]   // a span: a unit of work with a duration
fn checkout(order: &Order) {
    info!(total_cents, "order accepted");     // an event, inside the span: inherits order_id
}
```

A `Subscriber` decides what happens to them — printed as JSON, filtered,
exported to a tracing backend. Libraries only *emit*; the application picks
the subscriber.

### Which signal when

| | Cost | Good for |
|---|---|---|
| metrics | fixed per series | dashboards, alerts, trends |
| logs | per event | the details of one failure |
| traces | per request (sampled) | latency across services |

### A trace

```
trace 4bf92f…                                     0 ms        50 ms
frontend  http.server /checkout/{n}               ████████████████
frontend    http.client                           ██████
backend       http.server /price/{sku}             █████
backend         db.query                            ███
frontend    http.client                                  ██████
…
```

Each box is a span: an id, a parent id, a start and a duration. Services
share nothing but the `traceparent` header; the backend (Jaeger, Tempo,
Honeycomb) joins the spans by trace id.

## Common Pitfalls

1. **Unbounded label values** (user ids, raw paths) — every value is a new time series
2. **Logging secrets or personal data** — logs are copied, retained and widely readable
3. **Averages instead of percentiles** — the mean hides the slow 1%
4. **Alerting on causes** (CPU 80%) instead of symptoms users feel (errors, latency)
5. **Breaking the trace** — one service that doesn't forward `traceparent` splits it in two
6. **Per-service sampling decisions** — traces with holes
7. **`println!` in libraries** — nobody can filter or redirect it

## Running This Module

```bash
cargo run  -p m24-observability -- demo                  # your code
cargo test -p m24-observability-tests --features mine    # test your code
cargo run  -p m24-observability-solution -- demo         # logs, metrics, a trace tree, SLO maths
cargo run  -p m24-observability-solution -- serve        # frontend :3000, backend :3001
cargo test -p m24-observability-tests                    # 16 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (structured logging, Prometheus metrics,
distributed tracing, Grafana dashboards).

- **Traces are collected by a Layer you write**, not exported with the
  OpenTelemetry SDK. It's the same model (spans, ids, parents, attributes)
  without a collector to run; `tracing-opentelemetry` is the production route,
  and ANSWERS.md shows how it plugs in.
- **The Prometheus/Grafana stack in `deploy/` is optional and hasn't been run
  in this repository's checks** (Docker wasn't available); the generated
  dashboards and rules are checked by the tests.
- **Error tracking (Sentry) and APM** from the phase README's topic list
  aren't covered by an exercise.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[25 · Service Mesh](../25-service-mesh/)
