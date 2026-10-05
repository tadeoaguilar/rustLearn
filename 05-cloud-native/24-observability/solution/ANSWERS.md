# Answers · 24 Observability

## Exercise 2: Why mask in the application?

Once a value is written it's copied: to the log shipper's buffer, the
pipeline, the storage backend, backups, a developer's laptop during an
incident. Redacting downstream means every one of those copies — and every
future pipeline someone builds — must get it right; redacting at the source
means the value never leaves the process. Regulations (GDPR's data
minimisation) push the same way. Downstream scrubbing is a useful second line
of defence, not the first.

## Exercise 3: Why not label by the raw path?

Every distinct label combination is a separate time series that Prometheus
stores in memory and on disk. With `route="/price/A"`, `/price/B`, ... every
SKU, user id or random URL a scanner tries creates a new series: memory
grows without bound, queries slow down, and the server eventually falls over
("cardinality explosion"). The route template has a handful of values; put
per-request details in logs or traces instead.

## Exercise 6

**What does burn rate 14.4 for an hour cost?** Burn rate 1 spends the whole
30-day budget in 30 days (720 hours). At 14.4, one hour spends 14.4/720 =
**2% of the month's budget** — significant enough to wake someone up, and at
that rate the whole budget is gone in about two days. Burn rate 6 over 6 hours
is 5% of the budget: worth a ticket.

**Why the short window?** The long window alone keeps firing for a long time
after the problem is fixed: a one-hour average still contains the bad
minutes. Requiring the short window (1/12 of the long one) to also exceed the
threshold makes the alert stop soon after recovery, while the long window
keeps brief spikes from paging anyone.

## Bonus: Why must every service decide the same way?

A trace is only useful whole. If the frontend keeps a trace and the backend
drops its half, you see a request that spent 200 ms "somewhere". Propagating
the decision in the `sampled` flag (parent-based sampling) and deriving it
from the trace id when there's no parent makes every service agree without
talking to each other. Tail-based sampling — deciding after the trace
finishes, keeping all the slow and failed ones — needs a collector that sees
all spans first.

## Going to production with OpenTelemetry

The collector here and `tracing-opentelemetry` do the same job. In a real
service:

```rust
// Cargo.toml: opentelemetry, opentelemetry_sdk, opentelemetry-otlp, tracing-opentelemetry
let exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?;
let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .with_batch_exporter(exporter)
    .build();
let tracer = provider.tracer("my-service");
tracing_subscriber::registry()
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .with(tracing_subscriber::fmt::layer().json())
    .init();
```

and `opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new())`
for the `traceparent` header — the job `trace_requests` does by hand. (The
OpenTelemetry crates change their APIs often; check the versions' docs.)
