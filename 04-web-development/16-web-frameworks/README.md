# 16 · Web Frameworks

## Overview

Rust has several mature web frameworks; the three most used are **Axum**
(tower-based, part of the Tokio project), **Actix-web** (the long-time
performance leader) and **Rocket** (batteries included, attribute routing).
This module builds the same JSON API in Axum and Actix-web on one shared core,
so the comparison is about the frameworks and nothing else — then adds
server-rendered HTML, middleware and a load test.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Core | Business logic with no HTTP in it; `Clone` + `Arc` state |
| 2 | Axum | Extractors, `IntoResponse` errors, 201/204/404/422 |
| 3 | Actix-web | The same API; `ResponseError`; `actix_web::test` |
| 4 | Middleware | Request IDs and timing, in both frameworks |
| 5 | Templates & forms | askama, `Form<T>`, Post/Redirect/Get, escaping |
| 6 | Static files | `tower_http::services::ServeDir` |
| 7 | Load testing | p50/p99, requests per second, why benchmarks lie |
| Bonus | Rocket | Port it, compare |

## Key Concepts

### Axum vs Actix-web vs Rocket

| | Axum | Actix-web | Rocket |
|---|---|---|---|
| Handlers | async fns with extractor arguments | async fns with extractor arguments | attribute macros: `#[get("/notes/<id>")]` |
| State | `State<T>` (typed, checked at compile time) | `web::Data<T>` (looked up at runtime) | `&State<T>` |
| Errors | `impl IntoResponse` | `impl ResponseError` | `impl Responder` |
| Middleware | any tower `Layer` (huge ecosystem: tower-http) | `wrap`, `middleware::from_fn` | fairings |
| Runtime | Tokio | actix-rt: one single-threaded Tokio runtime per worker | Tokio |
| Feel | small, composable, explicit | mature, fast, a little more ceremony | most "batteries included" |

Honest summary: performance differences are small for typical apps (see
Exercise 7). Pick Axum for the tower ecosystem and Tokio alignment, Actix if
your team already knows it, Rocket for rapid, convention-driven development.

### Keep the framework at the edge

`NoteStore` doesn't know what HTTP is. Each framework adapter is ~60 lines of
glue: extract inputs, call the core, map `Result` to a status code. That's why
porting to another framework (the bonus) is an afternoon, and why the core is
tested without a server.

### Status codes that matter

| Situation | Code |
|---|---|
| Created a resource | **201** + `Location` header |
| Succeeded, nothing to return | **204** |
| Malformed request (bad JSON, `/notes/abc`) | 400 — the extractor rejects it |
| Well-formed but invalid (empty title) | **422** |
| Doesn't exist | 404 |
| Form submitted successfully | **303** See Other → GET the result |

### Templates

askama compiles `templates/*.html` into Rust at build time: a misspelt field
is a compile error, rendering is fast, and every `{{ value }}` is HTML-escaped
— so user input can't inject markup or scripts.

## Common Pitfalls

1. **Business logic in handlers** — untestable without a server, and locked to one framework
2. **`Json<T>` not last in Axum** — it consumes the body; the compiler error is long but says so
3. **200 for everything** — clients can't tell "created" from "nothing happened"
4. **Returning HTML after a successful POST** — a refresh resubmits the form; redirect with 303
5. **`ServeDir::new("static")`** — relative to the *working directory*, not the crate
6. **Benchmarking a debug build, or with the client on the same machine** — the numbers are mostly noise

## Running This Module

```bash
cargo run  -p m16-web-frameworks -- demo                         # your code
cargo test -p m16-web-frameworks-tests --features mine           # test your code
cargo run  -p m16-web-frameworks-solution -- demo                # both APIs exercised by a client
cargo run  -p m16-web-frameworks-solution -- axum                # serve on :3000 -- open http://127.0.0.1:3000/blog
cargo run  -p m16-web-frameworks-solution --release -- bench     # load-test both
cargo test -p m16-web-frameworks-tests                           # 14 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercise list in
[the phase README](../README.md) (an API in each framework, a blog with
templates, custom middleware, performance comparison).

- **Rocket** is covered in the comparison above and as an optional bonus, but
  not implemented in the workspace: it would add a third full dependency tree
  to every build for little extra learning. Axum and Actix-web are both
  implemented.
- **Templates and the stylesheet are provided** in `exercise/templates` and
  `exercise/static`. The exercise is the Rust side.
- **The skeleton's middleware passes requests through** instead of
  `todo!()`, so Exercises 2–3 can be tested before Exercise 4.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[17 · REST APIs](../17-rest-apis/)
