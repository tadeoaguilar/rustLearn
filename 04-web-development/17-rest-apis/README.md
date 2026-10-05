# 17 · REST APIs

## Overview

Module 16 was about *how* a framework handles requests. This one is about
designing an API other people can depend on: predictable status codes, one
error format, validation that reports every problem at once, pagination that
scales, versioning that doesn't break clients, documentation that can't go
stale, and protection against abuse.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | CRUD | 201 + `Location`, 204, 404 — and `version` for later |
| 2 | Validation & errors | `validator`, a `ValidatedJson` extractor, RFC 7807 problems |
| 3 | Pagination | page/per_page, filters, sorting, `X-Total-Count`, `Link` |
| 4 | Versioning | `/v2` with a new representation, `/v1` untouched |
| 5 | OpenAPI | generated with `utoipa` from the handlers themselves |
| 6 | Rate limiting | token bucket per client, 429 + `Retry-After` |
| 7 | CORS, negotiation | `CorsLayer`; JSON or CSV by `Accept`, 406 otherwise |
| 8 | Client SDK | typed errors, following `Link` headers |
| Bonus | ETags | 304 Not Modified, 412 Precondition Failed |

## Key Concepts

### Status codes

| Code | When |
|---|---|
| 200 | here's the result |
| 201 + `Location` | created |
| 204 | done, nothing to return |
| 304 | your cached copy is still current |
| 400 | malformed: bad JSON, unknown enum value, `page=0` |
| 404 | doesn't exist |
| 406 | can't produce any format you accept |
| 412 | your precondition (`If-Match`) failed |
| 422 | well-formed but breaks a rule |
| 429 | slow down (`Retry-After`) |

### One error format: problem+json

A client that can parse one error shape can handle every error your API
returns. RFC 9457 (formerly 7807) is that shape: `type` (stable, for code),
`title`, `status`, `detail` (for humans), plus extensions like `errors`.

### Validate at the boundary

`ValidatedJson<T>` is an extractor: by the time a handler runs, the input is
known to be valid. The handler can't forget to check, and every endpoint
reports errors identically.

### Pagination

Return the page, the total, and **links** — clients shouldn't build URLs
themselves. Always sort deterministically (tie-break on id) or items can
appear on two pages. For large or fast-changing data, prefer *keyset*
pagination (`?after=<last id>`) over offsets.

### Documentation from code

The OpenAPI document is generated from `#[utoipa::path]` annotations on the
handlers and `#[derive(ToSchema)]` on the types they use. Change a field and
the spec changes with it.

### Rate limiting

A token bucket allows bursts up to its capacity and a steady rate after that.
Key it on something the client can't trivially change (API key, user, IP),
always send `Retry-After`, and keep health checks unlimited. Across several
server instances the buckets must live in shared storage (Redis).

### Optimistic concurrency (bonus)

`ETag` + `If-Match` stop "lost updates": two clients read version 1, both
PUT, and the second silently overwrites the first. With `If-Match: "1-1"`
the second gets 412 and must re-read.

## Common Pitfalls

1. **200 with `{"error": ...}`** — breaks every HTTP client's error handling
2. **Different error shapes per endpoint** — clients end up special-casing each
3. **Validating in some handlers but not others** — use an extractor
4. **Unbounded `per_page`** — one request can fetch the whole table
5. **Non-deterministic ordering** — duplicate and missing items across pages
6. **Changing a field in place** — every existing client breaks; version it
7. **`allow_origin(Any)` with credentials** — browsers reject it; and it's rarely what you want

## Running This Module

```bash
cargo run  -p m17-rest-apis -- demo                       # your code
cargo test -p m17-rest-apis-tests --features mine         # test your code
cargo run  -p m17-rest-apis-solution -- demo              # a client exercises every feature
cargo run  -p m17-rest-apis-solution -- serve             # http://127.0.0.1:3000/openapi.json
cargo test -p m17-rest-apis-tests                         # 18 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the topics in
[the phase README](../README.md) (CRUD, validation with `validator`, OpenAPI
with `utoipa`, rate limiting, CORS, content negotiation, pagination, error
standards, a client SDK).

- **Swagger UI isn't bundled.** `utoipa-swagger-ui` downloads the UI from
  GitHub at *build* time, which would break offline builds. The document is
  served at `/openapi.json`; paste it into editor.swagger.io.
- **The skeleton's rate-limit middleware passes requests through** until you
  implement it.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[18 · Databases](../18-databases/)
