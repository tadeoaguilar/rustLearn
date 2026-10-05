# 25 · Service Mesh

## Overview

Every service needs timeouts, retries, load balancing, encryption and access
control. Writing them into every service, in every language, gives slightly
different and slightly wrong versions of each. A **service mesh** moves them
into proxies that sit next to every service and handle all its traffic (the
*data plane*), configured by a central *control plane*. This module builds the
data plane's features in Rust — the language of Linkerd's proxy — so the
mesh's configuration knobs stop being magic.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | L4 proxy | Copying bytes both ways; what a proxy sees without parsing |
| 2 | L7 routing | Route by path and header; hop-by-hop vs end-to-end headers |
| 3 | Retries | Idempotency, retryable outcomes, retry budgets |
| 4 | Traffic splits | Weighted, sticky routing; automated canary analysis |
| 5 | mTLS | A CA, SPIFFE identities, rustls on both sides |
| 6 | Authorization | Policies by workload identity |
| 7 | Load balancing | Power of two choices; outlier ejection with a cap |
| — | The sidecar | All of it in one proxy |
| Bonus | Manifests | HTTPRoute and Istio policies from the same data |

## Key Concepts

### Data plane and control plane

```
                 control plane (istiod, linkerd-destination, identity)
                   │ config, endpoints, certificates
        ┌──────────┴───────────┐
   ┌────▼────┐   mTLS     ┌────▼────┐
   │ proxy   │◀──────────▶│ proxy   │      data plane: every request
   ├─────────┤            ├─────────┤
   │ orders  │            │inventory│      the apps talk plain HTTP to localhost
   └─────────┘            └─────────┘
```

### Workload identity

Instead of trusting IP addresses (which pods reuse constantly), every
workload gets a short-lived certificate naming it —
`spiffe://cluster.local/ns/shop/sa/orders` — derived from its Kubernetes
service account. mTLS proves the identity on every connection; policies say
which identities may call what.

### When the mesh helps, and when it doesn't

| Mesh-level | Still the application's job |
|---|---|
| retries of idempotent calls, timeouts | knowing which calls are idempotent |
| mTLS between services | end-user authentication |
| canary routing by weight | feature flags, data migrations |
| golden metrics per route | business metrics, logs, traces' internal spans |

## Common Pitfalls

1. **Retrying in both the app and the mesh** — retry storms; pick one place
2. **Forwarding hop-by-hop headers** — broken connections, request smuggling risks
3. **Random (non-sticky) canary routing** — users flip between versions mid-session
4. **`PERMISSIVE` mTLS forever** — the migration mode accepts plaintext; switch to `STRICT`
5. **Authorizing by IP or namespace label alone** — IPs are reused; use identities
6. **Ejecting too many endpoints** — a cluster-wide problem becomes zero capacity
7. **Assuming the mesh makes calls reliable** — it can't make a non-idempotent call safe to retry

## Running This Module

```bash
cargo run  -p m25-service-mesh -- demo                  # your code
cargo test -p m25-service-mesh-tests --features mine    # test your code
cargo run  -p m25-service-mesh-solution -- demo         # every part, on local ports
cargo test -p m25-service-mesh-tests                    # 17 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (deploying with Linkerd, mTLS between services,
canary deployments, traffic policies).

- **The mesh's features are built, not installed.** Deploying Linkerd or Istio
  needs a cluster and isn't something tests can check; GETTING_STARTED.md has
  the optional commands, and the bonus generates the matching resources.
- **The sidecar is not production-grade**: it buffers whole bodies, opens no
  connection pools of its own beyond reqwest's, and doesn't speak HTTP/2 to
  upstreams or handle WebSocket upgrades. Linkerd's proxy is the reference
  for doing all of that.
- **Certificates live for the test run only**; real meshes rotate them
  automatically, typically every 24 hours.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 5 is complete. Continue with
[Phase 6 · Blockchain & Solana](../../06-blockchain-solana/).
