# Answers · 25 Service Mesh

## Exercise 1: What can't an L4 proxy do?

It sees a byte stream, not requests. So it can't route by path or header,
retry an individual request (it doesn't know where one ends), split traffic
per request (only per connection — and HTTP/2 or gRPC put many requests on one
long-lived connection, so per-connection balancing is badly uneven), record
per-route metrics or status codes, or authorize by method and path. What it
*can* do — and why meshes keep it as a fallback — is proxy any protocol,
including ones it doesn't understand, and terminate or originate TLS.

## Exercise 3: An outage with "retry 3 times" and no budget

Every client request becomes four upstream requests, so the failing service
receives **4× its normal load** exactly when it's least able to handle it —
and if each layer of a call chain retries, the multiplication compounds
(3 layers × 4 attempts = 64×). The service can't recover while the retries
keep it saturated. A budget (say 20% extra) caps the amplification at 1.2×:
during a real outage the budget is exhausted almost immediately and the proxy
fails fast instead, while occasional blips are still retried.

## Exercise 5: Why short-lived certificates?

A stolen key is useful only until its certificate expires, and revocation
(CRLs, OCSP) is unreliable in practice. With 24-hour certificates rotated
automatically, compromise windows are short and nobody ever handles a
certificate by hand — which is only practical because the mesh's control
plane automates issuing them.
