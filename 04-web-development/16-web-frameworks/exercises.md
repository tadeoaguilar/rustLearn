# Exercises: Web Frameworks

You'll build one small **notes API** twice — in Axum and in Actix-web — on top
of a single framework-agnostic core, then a server-rendered blog with
templates, and finally measure both servers under load.

**Setup**: everything is in `exercise/Cargo.toml` already: `axum`,
`actix-web`, `askama` (templates), `tower-http` (static files), `tokio`,
`serde`, `reqwest` (load testing). Templates live in `exercise/templates/`
and the stylesheet in `exercise/static/` — they're provided; you write Rust.

---

## Exercise 1: A Framework-Agnostic Core

**Difficulty**: Easy
**Time**: 40 minutes

**Learning Objectives**:
- Keep business logic out of HTTP handlers
- Share state cheaply between handlers (`Arc` inside a `Clone` type)

**Task**: implement `notes.rs`:
```rust
pub struct Note { pub id: u64, pub title: String, pub body: String, pub tags: Vec<String> }
pub struct NoteInput { pub title: String, #[serde(default)] pub body: String, #[serde(default)] pub tags: Vec<String> }
pub struct NoteFilter { pub tag: Option<String>, pub q: Option<String> }
pub enum NoteError { NotFound(u64), Invalid(String) }

#[derive(Clone, Default)]
pub struct NoteStore { notes: Arc<RwLock<BTreeMap<u64, Note>>>, next_id: Arc<AtomicU64> }
impl NoteStore {
    pub fn create(&self, input: NoteInput) -> Result<Note, NoteError>;
    pub fn get(&self, id: u64) -> Result<Note, NoteError>;
    pub fn list(&self, filter: &NoteFilter) -> Vec<Note>;
    pub fn update(&self, id: u64, input: NoteInput) -> Result<Note, NoteError>;   // replace
    pub fn delete(&self, id: u64) -> Result<(), NoteError>;
}
```

**Rules**: ids start at 1; the title is trimmed, required, and at most 120
characters; tags are lower-cased, trimmed, deduplicated and sorted; `tag`
filters exactly; `q` matches title or body, case-insensitively.

**Question**: why does every method take `&self`, even `create` and `delete`?

---

## Exercise 2: The API in Axum

**Difficulty**: Medium
**Time**: 45 minutes

**Learning Objectives**:
- Write handlers with extractors: `State`, `Path`, `Query`, `Json`
- Map domain errors to HTTP responses with `IntoResponse`

**Routes**:

| Method | Path | Success | Errors |
|---|---|---|---|
| GET | `/health` | 200 `{"status":"ok","framework":"axum"}` | |
| GET | `/notes?tag=&q=` | 200 list | |
| POST | `/notes` | **201** + `Location: /notes/{id}` | 422 invalid |
| GET | `/notes/{id}` | 200 | 404 |
| PUT | `/notes/{id}` | 200 | 404, 422 |
| DELETE | `/notes/{id}` | **204** | 404 |

Errors are JSON: `{"error": "note 7 not found"}`.

```rust
impl IntoResponse for NoteError { ... }

async fn get_note(State(store): State<NoteStore>, Path(id): Path<u64>) -> Result<Json<Note>, NoteError> {
    store.get(id).map(Json)
}
```

**Hints**:
- Axum 0.8 path syntax is `/notes/{id}` (0.7 used `/notes/:id`)
- `Json<T>` must be the *last* extractor: it consumes the body
- `axum_app::spawn(router, "127.0.0.1:0")` starts a server on a free port for tests

---

## Exercise 3: The Same API in Actix-web

**Difficulty**: Medium
**Time**: 45 minutes

**Learning Objectives**:
- See which ideas are universal (extractors, error mapping) and which are framework-specific
- Test without opening a socket using `actix_web::test`

**Task**: `actix_app.rs` — identical routes and responses, using
`web::Data<NoteStore>`, `web::Path`, `web::Query`, `web::Json`, and
`impl ResponseError for NoteError`.

```rust
pub fn configure(cfg: &mut web::ServiceConfig);                 // the route table
pub fn server(store: NoteStore, addr: &str, workers: usize)
    -> std::io::Result<(String, actix_web::dev::Server)>;        // base URL + server future
```

**Questions**:
- `HttpServer::new` takes a *closure*, not an `App`. Why?
- Can an Actix server run inside `#[tokio::main]`?

---

## Exercise 4: Middleware

**Difficulty**: Medium
**Time**: 40 minutes

**Task**: in both frameworks,
1. `x-request-id`: echo the client's header, or generate a UUID; return it on every response — errors included
2. `x-response-time-ms`: how long the request took, e.g. `"0.412"`

Axum: `axum::middleware::from_fn(request_id)` with
`async fn request_id(request: Request, next: Next) -> Response`.
Actix: `actix_web::middleware::from_fn(actix_request_id)`.

**Note**: the skeleton's middleware passes requests straight through, so the
other exercises work before you start this one.

---

## Exercise 5: Templates and Forms — a Blog

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Render HTML with askama (compile-time checked templates)
- Handle `application/x-www-form-urlencoded` forms
- Apply Post/Redirect/Get

**Routes** (Axum):

| Method | Path | Does |
|---|---|---|
| GET | `/blog` | list posts (`templates/index.html`) |
| GET | `/blog/new` | the form (`templates/new.html`) |
| POST | `/blog` | publish; **303** redirect to `/blog/{slug}`; on error **422** + the form again, with the user's text kept |
| GET | `/blog/{slug}` | one post (`templates/post.html`), body split into paragraphs on blank lines; 404 if missing |

```rust
#[derive(Template)]
#[template(path = "post.html")]
struct PostPage { post: Post }
```

Slugs: `"Hello, World! 2024"` → `"hello-world-2024"`; a repeated title gets
`-2`, `-3`, ...

**Check**: publish a post titled `Hello <b>World</b>` — the page must show
the tags as text, not bold. Who escaped them?

---

## Exercise 6: Static Files

**Difficulty**: Easy
**Time**: 15 minutes

Serve `static/` at `/static` with `tower_http::services::ServeDir`, so
`/static/style.css` returns 200 with `content-type: text/css`.

**Hint**: anchor the path at the crate root with
`concat!(env!("CARGO_MANIFEST_DIR"), "/static")`, so it works from any
working directory.

---

## Exercise 7: Load Testing Both Frameworks

**Difficulty**: Medium
**Time**: 40 minutes

Write a small load generator:

```rust
pub struct LoadReport { pub requests: usize, pub errors: usize, pub elapsed: Duration, pub p50: Duration, pub p99: Duration }
pub async fn load_test(url: &str, requests: usize, concurrency: usize) -> LoadReport;
```

`concurrency` tasks share one `reqwest::Client`, each sending its share of
requests back to back and recording latencies. Non-2xx responses are errors.
Then compare `GET /notes` on both servers, **in release mode**:

```bash
cargo run -p m16-web-frameworks --release -- bench
```

**Questions**: Which is faster on your machine, and by how much? Why can't
this benchmark tell you which framework is "faster" in production?

---

## Bonus Challenge: Port It to Rocket

**Difficulty**: Medium
**Time**: 60 minutes

Create a scratch crate *outside this workspace* with `rocket = { version =
"0.5", features = ["json"] }` and port the notes API: `#[get("/notes/<id>")]`
attribute routes, `&State<NoteStore>`, `Json<T>`, a `Responder` for
`NoteError`. Write down what Rocket made easier and what it made harder than
Axum and Actix-web.

---

## Check Your Understanding

- [ ] Keep business logic independent of the web framework
- [ ] Write Axum handlers with extractors and `IntoResponse` errors
- [ ] Write the same in Actix-web with `ResponseError`
- [ ] Add middleware in both
- [ ] Render templates and handle forms with Post/Redirect/Get
- [ ] Serve static files
- [ ] Load-test a server and read p50/p99

---

## Additional Resources

- [Axum docs](https://docs.rs/axum/) and [examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [Actix Web guide](https://actix.rs/docs/)
- [askama book](https://askama.readthedocs.io/)
- [Rocket guide](https://rocket.rs/guide/)
- [oha](https://github.com/hatoo/oha) — a real HTTP load generator
