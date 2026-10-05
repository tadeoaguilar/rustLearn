# Getting Started with 24 · Observability

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m24-observability-solution -- demo
```

prints JSON log lines, a trace across two services as a tree, the backend's
metrics, SLO arithmetic and the PromQL for a RED dashboard.

## What Is Already Here

```
24-observability/
├── README.md, exercises.md, GETTING_STARTED.md
├── deploy/                    # optional Prometheus + Grafana (Docker)
│   ├── compose.yaml
│   ├── prometheus/            #   prometheus.yml, rules.yaml (generated)
│   └── grafana/               #   provisioning, dashboards/*.json (generated)
├── exercise/src/              # ← YOUR WORKSPACE (package m24-observability)
│   ├── logging.rs             #   Ex 1, 2
│   ├── metrics.rs             #   Ex 3
│   ├── tracecontext.rs        #   Ex 4
│   ├── collector.rs           #   Ex 5: the Layer and render_tree
│   ├── services.rs            #   Ex 5: middleware and propagation (routers provided)
│   ├── slo.rs                 #   Ex 6
│   ├── dashboards.rs          #   Ex 7
│   ├── sampling.rs            #   bonus
│   └── capture.rs             #   provided: in-memory log capture
├── solution/                  # ← REFERENCE (m24-observability-solution) + ANSWERS.md
└── tests/                     # ← 16 tests (m24-observability-tests)
```

## The Commands You Need

```bash
cargo run  -p m24-observability -- demo
cargo test -p m24-observability-tests --features mine
cargo test -p m24-observability-tests --features mine ex5_
cargo test -p m24-observability-tests                    # the solution: always green
```

## See It Live

```bash
cargo run -p m24-observability-solution -- serve        # JSON logs on stdout
curl localhost:3000/checkout/3
curl -H 'traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01' localhost:3000/checkout/1
curl localhost:3000/metrics
curl localhost:3001/metrics
```

### With Prometheus and Grafana (Optional, Docker)

```bash
cargo run -p m24-observability-solution -- serve                              # terminal 1
docker compose -f 05-cloud-native/24-observability/deploy/compose.yaml up      # terminal 2
for i in $(seq 200); do curl -s localhost:3000/checkout/2 >/dev/null; done     # some traffic
```

- Prometheus: http://localhost:9090 — try `rate(http_requests_total[1m])`; *Alerts* shows the burn-rate rules
- Grafana: http://localhost:3002 (admin / admin) — dashboards "frontend -- RED" and "backend -- RED"

After changing `dashboards.rs`, regenerate the files:
`cargo run -p m24-observability-solution -- generate`.

## If You Get Stuck

1. **No log lines captured** — the subscriber must be active: `tracing::subscriber::with_default(sub, || ...)`.
2. **Fields nested under `"fields"`** — call `.flatten_event(true)`.
3. **Tracing tests see no spans** — a thread-local subscriber only sees its own thread; the tests use a current-thread runtime for that reason.
4. **Backend spans have their own trace id** — the frontend must send `traceparent`, and the middleware must use it.
5. **`route` is the raw path** — use `MatchedPath` from the request extensions.
6. **404s missing from the metrics** — the middleware must wrap the fallback too (`.layer`, not `.route_layer`).
7. Compare with `solution/src/` — same file and function names.
