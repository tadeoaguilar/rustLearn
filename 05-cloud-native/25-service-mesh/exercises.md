# Exercises: Service Mesh

A service mesh runs a proxy next to every service (a *sidecar*, or a
per-node proxy) and routes all traffic through them. The proxies give every
service, in any language, the same retries, timeouts, load balancing,
traffic splitting, mutual TLS and authorization — configured centrally, with
no application code. Linkerd's proxy is written in Rust. Here you build the
proxy features yourself, then see how Linkerd and Istio express them.

**Setup**: everything runs on local ports. `upstream.rs` (provided) starts
small HTTP services that are healthy, flaky, slow or broken on demand.
Deploying a real mesh (Linkerd or Istio on a cluster) is optional — see
GETTING_STARTED.md.

---

## Exercise 1: An L4 Proxy

**Difficulty**: Easy
**Time**: 25 minutes

`run_tcp_proxy(listener, upstream, stats)`: for every connection, connect to
the upstream and copy both ways (`tokio::io::copy_bidirectional`); count
connections, bytes each way and upstream connection failures (then just close
the client's connection).

**Question**: what can an L4 proxy not do that an L7 one can?

---

## Exercise 2: L7 Routing and Headers

**Difficulty**: Medium
**Time**: 45 minutes

`routing.rs`:
- `RouteTable::route(path, headers)` — routes with a matching header
  condition win; then the longest prefix. Prefixes match whole segments
  (`/api` matches `/api/x`, not `/apix`).
- `strip_hop_by_hop` — `Connection`, `Keep-Alive`, `Transfer-Encoding`,
  `Upgrade`, `TE`, `Trailer`, `Proxy-*`, **and any header `Connection` names**
- `upstream_headers(incoming, client_ip)` — hop-by-hop and `Host` removed,
  client appended to `X-Forwarded-For`, `X-Request-Id` added if missing

---

## Exercise 3: Retries With a Budget

**Difficulty**: Medium
**Time**: 40 minutes

- `is_retryable(method, outcome)`: idempotent methods only; connection
  failures, 502 and 503 — not 500 (a bug), not timeouts (still running?)
- `RetryBudget::new(ratio, min_per_window, window)`: `try_retry` succeeds
  while retries in the window < `min + ratio × requests in the window`

**Question**: an outage makes every request fail. With "retry 3 times" and
no budget, what happens to the load on the failing service?

---

## Exercise 4: Traffic Splits and Canaries

**Difficulty**: Medium
**Time**: 45 minutes

- `fnv1a(key)` — the 64-bit FNV-1a hash
- `pick_weighted(weights, sticky_key, roll)` — proportional; with a key, the
  choice depends only on its hash (the same user always gets the same version)
- `CanaryController::new(steps, max_error_rate, min_requests)`:
  `evaluate(requests, errors)` → `Rollback` as soon as the error rate is too
  high, `Hold` until there's enough traffic, then `Promote(next weight)`, and
  finally `Complete`

---

## Exercise 5: Mutual TLS

**Difficulty**: Hard
**Time**: 75 minutes

`mtls.rs` with rcgen and rustls:
- `Pki::new()` — a CA; `issue(spiffe_id, dns_name, expired)` — a certificate
  with the SPIFFE id as a URI SAN (plus a DNS name for servers)
- `server_config` — *requires* client certificates signed by the CA;
  `client_config` — trusts the CA, presents a certificate if given
- `spiffe_id(cert)` — read the URI SAN back (x509-parser)
- `serve` / `call` — a one-line request/response service over mTLS that
  greets the caller by identity

Rejected: no client certificate, another CA's certificate, an expired one, a
server whose name doesn't match.

---

## Exercise 6: Authorization by Identity

**Difficulty**: Easy
**Time**: 25 minutes

`Policy { rules }`, each rule: caller (`spiffe://.../sa/orders`, or a prefix
ending in `*`), methods (empty = any), path prefix. `authorize(identity,
method, path)` → `Allow`, `Deny` (default), or `Unauthenticated` with no
identity. `serve` enforces it (401 / 403).

---

## Exercise 7: Load Balancing and Outlier Detection

**Difficulty**: Hard
**Time**: 50 minutes

`Balancer`:
- `pick(now, a, b)` — power of two choices among non-ejected endpoints
  (`a`, `b`: random numbers from the caller) — the one with fewer requests
  in flight
- `finish(index, success, now)` — N consecutive failures eject the endpoint
  for `ejection_time`, but never more than half the endpoints at once

**The sidecar** (`proxy.rs`, Exercises 2–4 and 7 together): route → cluster
(weighted, sticky by `x-user-id`) → endpoint → forward with the cluster's
timeout (504) → retry if allowed (budget, max retries) → answer with
`x-mesh-cluster` and `x-mesh-attempts` headers. Unreachable: 502; no route:
404; no healthy endpoint: 503.

---

## Bonus Challenge: Mesh Manifests

**Difficulty**: Easy
**Time**: 25 minutes

`manifests.rs` — the same configuration for real meshes:
- a Gateway API `HTTPRoute` with weighted `backendRefs` (Linkerd, Istio)
- an Istio `AuthorizationPolicy` from a `Policy` (principals are SPIFFE ids
  without `spiffe://`)
- an Istio `PeerAuthentication` requiring mTLS

---

## Check Your Understanding

- [ ] Explain sidecars, the data plane and the control plane
- [ ] Explain L4 vs L7 proxying
- [ ] Forward HTTP correctly (hop-by-hop headers, X-Forwarded-For)
- [ ] Retry safely: idempotency, budgets
- [ ] Run a canary with sticky, weighted routing and automatic rollback
- [ ] Set up mutual TLS and explain SPIFFE identities
- [ ] Authorize by workload identity instead of IP address
- [ ] Explain P2C load balancing and outlier ejection

---

## Additional Resources

- [Linkerd docs](https://linkerd.io/2/overview/) and [linkerd2-proxy](https://github.com/linkerd/linkerd2-proxy) (Rust)
- [Istio: traffic management](https://istio.io/latest/docs/concepts/traffic-management/) and [security](https://istio.io/latest/docs/concepts/security/)
- [SPIFFE concepts](https://spiffe.io/docs/latest/spiffe-about/spiffe-concepts/)
- [Gateway API: HTTPRoute traffic splitting](https://gateway-api.sigs.k8s.io/guides/traffic-splitting/)
- [rustls](https://docs.rs/rustls), [rcgen](https://docs.rs/rcgen)
- [The power of two random choices](https://www.eecs.harvard.edu/~michaelm/postscripts/mythesis.pdf) — Mitzenmacher
