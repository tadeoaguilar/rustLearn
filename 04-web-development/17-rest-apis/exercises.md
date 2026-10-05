# Exercises: REST APIs

You'll build one **tasks API** to production conventions: correct status
codes, validation, a standard error format, pagination, versioning, generated
OpenAPI docs, rate limiting, CORS, content negotiation, and a typed client.

**Setup**: `axum`, `tower-http`, `serde`, `validator`, `utoipa`, `reqwest` and
`thiserror` are in `exercise/Cargo.toml`.

---

## Exercise 1: CRUD With the Right Status Codes

**Difficulty**: Medium
**Time**: 45 minutes

```rust
pub enum Status { Todo, InProgress, Done }          // JSON: "todo", "in_progress", "done"
pub struct Task { pub id: u64, pub title: String, pub description: Option<String>,
                  pub status: Status, pub priority: u8, pub version: u32 }
pub struct TaskInput { title, description, status (default todo), priority (default 3) }
```

| Method | Path | Success |
|---|---|---|
| GET | `/v1/tasks` | 200, a page (Exercise 3) |
| POST | `/v1/tasks` | **201** + `Location` |
| GET | `/v1/tasks/{id}` | 200 / 404 |
| PUT | `/v1/tasks/{id}` | 200 (replaces; `version` + 1) / 404 |
| DELETE | `/v1/tasks/{id}` | **204** / 404 |

`TaskStore::with_generated(n)` creates `Task 1` … `Task n` with priorities
cycling `i % 5 + 1` and statuses cycling `[todo, in_progress, done][i % 3]` —
the tests rely on that.

---

## Exercise 2: Validation and a Standard Error Format

**Difficulty**: Medium
**Time**: 50 minutes

1. Declare rules with `validator`: title 1–100 chars, description ≤ 1000,
   priority 1–5.
   ```rust
   #[derive(Deserialize, Validate)]
   pub struct TaskInput {
       #[validate(length(min = 1, max = 100, message = "must be 1 to 100 characters"))]
       pub title: String,
       ...
   }
   ```
2. Every error — 400, 404, 422, 429 … — is an RFC 7807 problem, served as
   `application/problem+json`:
   ```json
   {"type": "https://rustlearn.dev/problems/validation", "title": "Validation failed",
    "status": 422, "detail": "...", "errors": {"priority": ["must be between 1 and 5"]}}
   ```
3. Write an extractor `ValidatedJson<T>` (`impl FromRequest`) that parses JSON
   (400 on failure) and validates it (422 with per-field `errors`). Handlers
   then only ever see valid input.
4. Unknown routes return a problem 404 (`Router::fallback`).

---

## Exercise 3: Pagination, Filtering, Sorting

**Difficulty**: Medium
**Time**: 45 minutes

`GET /v1/tasks?page=2&per_page=10&status=done&sort=-priority`

- `page` from 1 (default 1); `per_page` 1–100 (default 20); invalid → 400
- `status` filters; `sort` is `id`, `priority` or `title`, `-` for descending; ties broken by id
- body: `{"data": [...], "page": 2, "per_page": 10, "total": 23}`
- headers: `X-Total-Count: 23` and an RFC 8288 `Link` header with `next`,
  `prev`, `first`, `last` — keeping the filter and sort parameters

```
Link: </v1/tasks?page=3&per_page=10>; rel="next", </v1/tasks?page=1&per_page=10>; rel="prev", ...
```

**Question**: offset pagination (`page=N`) has a problem when items are
inserted while a client is paging. What is it, and what's the alternative?

---

## Exercise 4: Versioning

**Difficulty**: Easy
**Time**: 30 minutes

`/v2/tasks` and `/v2/tasks/{id}` return a *different representation*:
- `priority` is a label: 1 → `"lowest"`, 2 `"low"`, 3 `"medium"`, 4 `"high"`, 5 `"highest"`
- each task has `"links": {"self": "/v2/tasks/5"}`
- no `version` field

v1 keeps working unchanged. Implement `impl From<Task> for TaskV2`.

**Question**: path versioning (`/v2/...`) vs a header (`Accept: application/vnd.tasks.v2+json`) — pros and cons?

---

## Exercise 5: OpenAPI Documentation

**Difficulty**: Medium
**Time**: 40 minutes

Generate an OpenAPI 3 document with `utoipa` and serve it at
`/openapi.json`:
- `#[derive(ToSchema)]` on `Task`, `TaskInput`, `Status`, `Problem`, `TaskV2`, `Page<T>`
- `#[derive(IntoParams)]` on the query struct
- `#[utoipa::path(get, path = "/v1/tasks/{id}", responses(...))]` on each handler
- `#[derive(OpenApi)] #[openapi(paths(...), components(schemas(...)))] struct ApiDoc;`

Paste the JSON into <https://editor.swagger.io> to browse it.

---

## Exercise 6: Rate Limiting

**Difficulty**: Hard
**Time**: 50 minutes

A token bucket per client (`x-api-key` header, or `"anonymous"`): capacity
`capacity`, refilled at `refill_per_sec` tokens per second.

- allowed: add `x-ratelimit-limit` and `x-ratelimit-remaining`
- over the limit: **429**, a problem body, `Retry-After: <seconds>` (rounded up, at least 1)
- applies to `/v1` and `/v2` only — not `/health` or `/openapi.json`

Write `check_at(&self, key, now: Instant) -> Decision` with the clock as a
parameter, so the logic is testable without sleeping.

---

## Exercise 7: CORS and Content Negotiation

**Difficulty**: Medium
**Time**: 40 minutes

1. **CORS** with `tower_http::cors::CorsLayer`: allow only configured origins,
   methods GET/POST/PUT/DELETE, headers `content-type`, `if-match`,
   `if-none-match`; expose `link`, `etag`, `x-total-count` (or browser JS can't
   read them).
2. **Content negotiation** for `GET /v1/tasks`: `Accept: text/csv` returns
   `id,title,status,priority` CSV; JSON otherwise; honour q-values
   (`text/csv;q=0.5, application/json` → JSON); only unsupported types → **406**.

---

## Exercise 8: A Client SDK

**Difficulty**: Medium
**Time**: 45 minutes

```rust
pub struct TasksClient { ... }
impl TasksClient {
    pub fn new(base: &str) -> Self;
    pub fn with_api_key(self, key: &str) -> Self;
    pub async fn create(&self, input: &TaskInput) -> Result<Task, ClientError>;
    pub async fn get(&self, id: u64) -> Result<Task, ClientError>;
    pub async fn replace(&self, id: u64, input: &TaskInput) -> Result<Task, ClientError>;
    pub async fn delete(&self, id: u64) -> Result<(), ClientError>;
    pub async fn list_all(&self, per_page: u32) -> Result<Vec<Task>, ClientError>;  // follows Link: rel="next"
}
pub enum ClientError { Api(Problem), RateLimited { retry_after_secs: u64 }, Http(reqwest::Error) }
```

---

## Bonus Challenge: ETags and Conditional Requests

**Difficulty**: Hard
**Time**: 45 minutes

- `GET`/`POST`/`PUT` responses carry `ETag: "<id>-<version>"`
- `GET` with a matching `If-None-Match` → **304 Not Modified**, empty body
- `PUT` with an `If-Match` that doesn't match the current version → **412
  Precondition Failed** (optimistic concurrency: refuse to overwrite a change
  the client never saw)

---

## Check Your Understanding

- [ ] Choose the right status code for create, delete, invalid, missing, conflict
- [ ] Validate input declaratively and report every field error
- [ ] Return RFC 7807 problems for every error
- [ ] Paginate with totals and `Link` headers
- [ ] Version an API without breaking old clients
- [ ] Generate OpenAPI from code
- [ ] Rate-limit per client, with `Retry-After`
- [ ] Configure CORS and content negotiation
- [ ] Wrap an API in a typed client

---

## Additional Resources

- [RFC 9457: Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457)
- [RFC 8288: Web Linking](https://www.rfc-editor.org/rfc/rfc8288)
- [utoipa](https://docs.rs/utoipa/) · [validator](https://docs.rs/validator/) · [tower-http CORS](https://docs.rs/tower-http/latest/tower_http/cors/)
- [Zalando RESTful API Guidelines](https://opensource.zalando.com/restful-api-guidelines/)
