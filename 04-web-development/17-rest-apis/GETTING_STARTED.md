# Getting Started with 17 · REST APIs

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m17-rest-apis-solution -- demo
```

## What Is Already Here

```
17-rest-apis/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/          # ← YOUR WORKSPACE (package m17-rest-apis)
│   ├── model.rs           #   Ex 1
│   ├── problem.rs         #   Ex 2: problem+json
│   ├── validation.rs      #   Ex 2: ValidatedJson extractor
│   ├── pagination.rs      #   Ex 3
│   ├── api.rs             #   Ex 1-4, 7, bonus: the routes
│   ├── openapi.rs         #   Ex 5
│   ├── rate_limit.rs      #   Ex 6 (passes through until implemented)
│   ├── client.rs          #   Ex 8
│   └── main.rs            #   runner: demo / serve
├── solution/              # ← REFERENCE (m17-rest-apis-solution) + ANSWERS.md
└── tests/                 # ← 18 tests (m17-rest-apis-tests)
```

## The Commands You Need

```bash
cargo run  -p m17-rest-apis -- demo
cargo test -p m17-rest-apis-tests --features mine
cargo test -p m17-rest-apis-tests --features mine ex3_     # one exercise
cargo run  -p m17-rest-apis-solution -- serve              # http://127.0.0.1:3000
cargo test -p m17-rest-apis-tests                          # always green
```

## Explore the API With curl

```bash
curl -s 'localhost:3000/v1/tasks?per_page=3&sort=-priority' -D - | head -20   # see Link + X-Total-Count
curl -s localhost:3000/v1/tasks -H 'accept: text/csv'
curl -s localhost:3000/v2/tasks/1 | jq
curl -s -X POST localhost:3000/v1/tasks -H 'content-type: application/json' -d '{"title":"","priority":7}' | jq
curl -s localhost:3000/openapi.json | jq '.paths | keys'

# ETags
curl -si localhost:3000/v1/tasks/1 | grep -i etag
curl -si localhost:3000/v1/tasks/1 -H 'if-none-match: "1-1"' | head -1        # 304

# Rate limit: capacity 20 in the `serve` config
for i in $(seq 25); do curl -s -o /dev/null -w '%{http_code} ' localhost:3000/v1/tasks/1 -H 'x-api-key: me'; done; echo

# CORS preflight
curl -si -X OPTIONS localhost:3000/v1/tasks -H 'origin: http://localhost:5173' -H 'access-control-request-method: POST' | grep -i access-control
```

## If You Get Stuck

1. **Your extractor isn't used** — `impl<T, S> FromRequest<S> for ValidatedJson<T>` needs `T: DeserializeOwned + Validate` and `S: Send + Sync`.
2. **`content-type` is `application/json` for errors** — `axum::Json` sets it; overwrite the header with `application/problem+json`.
3. **utoipa: `the trait ToSchema is not implemented`** — every type in a `body =` or `schemas(...)` needs `#[derive(ToSchema)]`, including nested ones (`Status`, `Links`).
4. **CORS headers missing in the browser but present in curl** — the browser only sends `Origin` cross-origin; and only listed origins get `access-control-allow-origin`.
5. Compare against `solution/src/` — same file and function names.
