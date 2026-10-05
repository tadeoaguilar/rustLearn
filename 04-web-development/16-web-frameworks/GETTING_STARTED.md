# Getting Started with 16 · Web Frameworks

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m16-web-frameworks-solution -- demo
```

## What Is Already Here

```
16-web-frameworks/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/                 # ← YOUR WORKSPACE (package m16-web-frameworks)
│   ├── templates/            #   askama templates (provided)
│   ├── static/style.css      #   (provided)
│   └── src/
│       ├── notes.rs          #   Ex 1: the core
│       ├── axum_app.rs       #   Ex 2
│       ├── actix_app.rs      #   Ex 3
│       ├── middleware.rs     #   Ex 4 (passes requests through until you implement it)
│       ├── blog.rs           #   Ex 5 and 6
│       ├── loadtest.rs       #   Ex 7
│       └── main.rs           #   runner: demo / axum / actix / bench
├── solution/                 # ← REFERENCE (m16-web-frameworks-solution) + ANSWERS.md
└── tests/                    # ← 14 tests (m16-web-frameworks-tests)
```

## The Commands You Need

```bash
# Your code
cargo run  -p m16-web-frameworks -- demo
cargo test -p m16-web-frameworks-tests --features mine
cargo test -p m16-web-frameworks-tests --features mine ex2_      # one exercise

# Serve for real, then use curl or a browser
cargo run -p m16-web-frameworks-solution -- axum     # http://127.0.0.1:3000
cargo run -p m16-web-frameworks-solution -- actix    # http://127.0.0.1:3001

# Load test (always --release)
cargo run -p m16-web-frameworks-solution --release -- bench

cargo test -p m16-web-frameworks-tests               # always green
```

## Try the API With curl

```bash
curl -s localhost:3000/notes | jq
curl -i -X POST localhost:3000/notes -H 'content-type: application/json' -d '{"title":"curl note","tags":["CLI"]}'
curl -s 'localhost:3000/notes?tag=cli' | jq
curl -i -X PUT localhost:3000/notes/4 -H 'content-type: application/json' -d '{"title":"edited"}'
curl -i -X DELETE localhost:3000/notes/4
curl -i localhost:3000/notes/4                      # 404, JSON error, x-request-id header
curl -i localhost:3000/notes/abc                    # 400 from the Path extractor
```

The same commands work against Actix on port 3001. For the blog, open
`http://127.0.0.1:3000/blog` in a browser and write a post.

## Testing Actix Without a Server

```rust
let app = actix_web::test::init_service(App::new().app_data(data).configure(configure)).await;
let resp = actix_web::test::call_service(&app, TestRequest::get().uri("/notes").to_request()).await;
```

Use `#[actix_web::test]` for these tests. And don't `use actix_web::test;` in a
file that also has plain `#[test]` functions — it shadows the attribute (alias
it: `use actix_web::test as atest;`).

## If You Get Stuck

1. **A long error about `Handler<_, _>` not implemented** — an argument isn't an extractor, or `Json` isn't last, or the return type isn't `IntoResponse`.
2. **404 for a route you defined** — Axum 0.8 uses `{id}`, not `:id`.
3. **askama: `template not found`** — templates must be in `templates/` next to `Cargo.toml`.
4. **Static files 404 only when run from another directory** — use `env!("CARGO_MANIFEST_DIR")`.
5. Compare against `solution/src/` — same file and function names.
