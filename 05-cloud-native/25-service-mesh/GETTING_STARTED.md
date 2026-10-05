# Getting Started with 25 · Service Mesh

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m25-service-mesh-solution -- demo
```

runs a TCP proxy, then an L7 sidecar in front of several upstreams (an 80/20
sticky split, a header-routed canary, a flaky service retried to success, a
slow one timed out), a canary rollout and rollback, and mTLS calls that are
allowed, forbidden, or rejected at the handshake.

## What Is Already Here

```
25-service-mesh/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m25-service-mesh)
│   ├── tcp_proxy.rs           #   Ex 1
│   ├── routing.rs             #   Ex 2
│   ├── resilience.rs          #   Ex 3
│   ├── split.rs               #   Ex 4
│   ├── mtls.rs                #   Ex 5
│   ├── policy.rs              #   Ex 6
│   ├── balancer.rs            #   Ex 7
│   ├── proxy.rs               #   the sidecar (Ex 2-4, 7 together)
│   ├── manifests.rs           #   bonus
│   └── upstream.rs            #   provided: test upstreams
├── solution/                  # ← REFERENCE (m25-service-mesh-solution) + ANSWERS.md
└── tests/                     # ← 17 tests (m25-service-mesh-tests)
```

## The Commands You Need

```bash
cargo run  -p m25-service-mesh -- demo
cargo test -p m25-service-mesh-tests --features mine
cargo test -p m25-service-mesh-tests --features mine ex5_
cargo test -p m25-service-mesh-tests --features mine sidecar_
cargo test -p m25-service-mesh-tests                    # the solution: always green
```

## A Real Mesh (Optional)

With a cluster (kind, Docker Desktop's Kubernetes...) and the
[Linkerd CLI](https://linkerd.io/2/getting-started/):

```bash
linkerd check --pre && linkerd install --crds | kubectl apply -f - && linkerd install | kubectl apply -f -
linkerd check
kubectl create namespace shop
kubectl annotate namespace shop linkerd.io/inject=enabled      # every new pod gets a proxy
# deploy two versions of a service (e.g. module 22's App, or `kubectl create deployment`), then:
cargo run -q -p m25-service-mesh-solution -- demo | sed -n '/^{/,$p' > /tmp/route.json   # the HTTPRoute
kubectl apply -f /tmp/route.json
linkerd viz install | kubectl apply -f - && linkerd viz stat deploy -n shop           # golden metrics per deployment
```

Istio's equivalents: `istioctl install`, `kubectl label namespace shop
istio-injection=enabled`, and the `AuthorizationPolicy` / `PeerAuthentication`
from `manifests.rs`.

## If You Get Stuck

1. **The TCP test hangs** — the client half-closes (`shutdown`); `copy_bidirectional` handles that, a pair of hand-written loops often doesn't.
2. **`x-secret-hop` still arrives upstream** — headers *named in* `Connection` are hop-by-hop too.
3. **Sticky routing isn't sticky** — with a key, don't use the random roll at all.
4. **"no process-level CryptoProvider"** — build configs with `builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))`.
5. **The no-certificate client "connects"** — with TLS 1.3 the server's rejection arrives on the first read; `call` must read the reply.
6. **Retries happen for POST, or for timeouts** — check `is_retryable` first, then the budget.
7. Compare with `solution/src/` — same file and function names.
