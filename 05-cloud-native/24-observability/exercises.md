# Exercises: Observability

Observability is being able to answer new questions about a running system
from the outside. Three signals do most of the work:

| Signal | Answers | Here |
|---|---|---|
| **logs** | what happened to this request? | `tracing` events, as JSON |
| **metrics** | how much, how fast, how often — for all requests? | Prometheus counters and histograms |
| **traces** | where did the time go, across services? | spans, `traceparent`, a collector |

**Setup**: the exercises use `tracing`, `tracing-subscriber`,
`prometheus-client` and axum. Everything is checked in-process: logs are
captured in memory (`capture.rs`, provided), spans by your own collector.
Prometheus and Grafana (Exercise 7) are optional, with Docker.

---

## Exercise 1: Structured Logs

**Difficulty**: Easy
**Time**: 40 minutes

**Learning Objectives**:
- Log events with fields, not formatted strings
- Give events context with spans

1. `json_subscriber(writer, directives)` — a `tracing_subscriber::fmt()`
   subscriber writing JSON lines, fields flattened to the top level, with the
   current span included and an `EnvFilter`.
2. `checkout(&Order)` — add `#[instrument]` with a span named `checkout` and
   the fields `order_id`, `customer` (masked, Exercise 2) and `items`. Log
   `info!(total_cents, "order accepted")`, or `warn!(reason = "empty", ...)`.

```json
{"level":"INFO","message":"order accepted","total_cents":9248,
 "span":{"name":"checkout","order_id":"o-42","customer":"a***@example.com","items":2}, ...}
```

---

## Exercise 2: Filtering and Redaction

**Difficulty**: Easy
**Time**: 25 minutes

- Filter per module: `"debug,<crate>::logging::noisy=warn"` silences the
  noisy module's debug output (use `noisy::TARGET`).
- `mask_email("alice@example.com") == "a***@example.com"`; anything else → `"***"`.
- `Redacted<T>` whose `Debug` and `Display` print `***`.

**Question**: why mask in the application instead of in the log pipeline?

---

## Exercise 3: Prometheus Metrics

**Difficulty**: Medium
**Time**: 50 minutes

`Metrics` with `prometheus-client`:

| Metric | Type | Labels |
|---|---|---|
| `http_requests_total` | counter | method, route, status |
| `http_request_duration_seconds` | histogram (`BUCKETS`) | method, route |
| `http_requests_in_flight` | gauge | — |

`track` — axum middleware recording every request, labelled by the
**route template** (`MatchedPath`: `/price/{sku}`), `"unmatched"` when no
route matched. `metrics_handler` serves `render()` with the OpenMetrics
content type.

**Question**: why is labelling by the raw path (`/price/A`) dangerous?

---

## Exercise 4: W3C Trace Context

**Difficulty**: Medium
**Time**: 30 minutes

```rust
TraceParent::parse("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
TraceParent { trace_id: u128, span_id: u64, sampled: bool }
.header()  .child()  ::new_root(sampled)  .trace_id_hex()  .span_id_hex()
```

Reject: other versions (`ff` especially), wrong lengths, upper-case or non-hex
digits, all-zero ids.

---

## Exercise 5: Distributed Tracing

**Difficulty**: Very Hard
**Time**: 90 minutes

1. **`SpanCollector`**, a `tracing_subscriber::Layer`:
   - `on_new_span`: read the span's fields; take `trace_id`, `span_id`,
     `parent_id`, `service` from them if present, otherwise inherit trace and
     service from the parent span and generate a span id; store it in the
     span's extensions. Spans with no trace at all aren't collected.
   - `on_record`: add later fields (`http.status`)
   - `on_close`: push a `FinishedSpan` with its duration
2. **`trace_requests`** middleware: continue the incoming `traceparent` (or
   start a trace), open an `http.server` span with the ids, method and route,
   record `http.status` at the end; put the current `TraceParent` in the
   request extensions.
3. **`checkout`**: for each call to the backend, a `http.client` span with a
   new span id that also goes out in the `traceparent` header.
4. **`render_tree`**: the spans of one trace as an indented tree.

```text
frontend http.server http.route=/checkout/{n}
  frontend http.client
    backend http.server http.route=/price/{sku}
      backend db.query
```

---

## Exercise 6: SLOs and Burn-Rate Alerts

**Difficulty**: Medium
**Time**: 40 minutes

```rust
Slo { objective: 0.999 }.error_budget()          // 0.001
Slo::availability(Counts { good, total })
slo.burn_rate(counts)                             // error rate / error budget
slo.budget_remaining(counts_for_the_window)
slo.evaluate(last_5m, last_30m, last_1h, last_6h) // Page / Ticket / None
```

Page when the burn rate exceeds 14.4 over 1 hour **and** 5 minutes; ticket
when it exceeds 6 over 6 hours **and** 30 minutes.

**Questions**: what does burn rate 14.4 for one hour cost a 30-day budget?
Why the short window?

---

## Exercise 7: Dashboards and Alerts as Code

**Difficulty**: Medium
**Time**: 40 minutes

- `red_queries(job)` — PromQL for request rate by route, error ratio (5xx),
  and p99 latency (`histogram_quantile`)
- `red_dashboard(job)` — a Grafana dashboard JSON with one time-series panel each
- `burn_rate_rules(job, objective)` — a Prometheus rules file with Exercise 6's
  two alerts in PromQL

With Docker, `deploy/compose.yaml` runs Prometheus and Grafana against
`-- serve`, with the generated dashboards and rules loaded (GETTING_STARTED.md).

---

## Bonus Challenge: Sampling

**Difficulty**: Easy
**Time**: 20 minutes

`should_sample(parent, trace_id, ratio)`: follow the parent's sampled flag
if there is one; otherwise sample when the low 64 bits of the trace id are
below `ratio × u64::MAX`. Why must every service reach the same decision?

---

## Check Your Understanding

- [ ] Log events with fields and spans; filter by module
- [ ] Keep personal data out of logs
- [ ] Instrument a service with RED metrics without exploding cardinality
- [ ] Propagate a trace across services with `traceparent`
- [ ] Explain spans, parents and how a trace is assembled
- [ ] Define an SLO and alert on burn rate
- [ ] Keep dashboards and alerts in version control

---

## Additional Resources

- [tracing](https://docs.rs/tracing) and [tracing-subscriber](https://docs.rs/tracing-subscriber)
- [prometheus-client](https://docs.rs/prometheus-client) and [Prometheus naming conventions](https://prometheus.io/docs/practices/naming/)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [OpenTelemetry for Rust](https://opentelemetry.io/docs/languages/rust/) and [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry)
- [Google SRE Workbook: Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/)
