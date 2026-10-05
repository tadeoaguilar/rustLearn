# Exercises: Containerization

A container runs **one process**, with no terminal, configured by
**environment variables**, stopped with **SIGTERM**, and watched by
**probes**. The exercises build a small HTTP service that behaves well in
that world, then the images and Compose file that run it — plus two tools,
a Dockerfile linter and a Compose dependency checker, so the rules are
checked by code.

**Setup**: everything is tested without Docker. To build and run the images
(Exercise 8, optional), install Docker Desktop or another engine with
BuildKit. Every `docker` command runs from the **repository root**.

---

## Exercise 1: Configuration From the Environment

**Difficulty**: Easy
**Time**: 40 minutes

**Learning Objectives**:
- Apply twelve-factor rule III: config in the environment, not the image
- Report every configuration error at once
- Read secrets from files and keep them out of logs

**Task**: `config.rs`

| Variable | Default | Notes |
|---|---|---|
| `HOST` | `0.0.0.0` | why not `127.0.0.1` in a container? |
| `PORT` | `8080` | 1–65535 |
| `LOG_LEVEL` | `info` | error, warn, info, debug, trace — any case |
| `DATABASE_URL` | none | |
| `API_TOKEN` *or* `API_TOKEN_FILE` | none | both set is an error; strip the file's trailing newline |
| `DRAIN_DELAY_MS` | `5000` | |
| `SHUTDOWN_TIMEOUT_MS` | `20000` | |

```rust
pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigErrors>;
```

- an empty value counts as unset
- collect **all** errors: `PORT=eighty LOG_LEVEL=loud` reports both
- `Secret`'s `Debug` prints `Secret(***)`

**Why `lookup` and not `std::env::var`?** Tests run in parallel threads, and
the environment is shared by the whole process: tests changing it race each
other (`std::env::set_var` is `unsafe` since edition 2024 for this reason).

---

## Exercise 2: Liveness, Readiness, Startup

**Difficulty**: Medium
**Time**: 30 minutes

**Learning Objectives**:
- Know what each probe means to an orchestrator
- Keep dependencies out of liveness

**Task**: `health.rs` — `Health` holds: started, shutting down, wedged, and a
map of dependencies (name → up). Each probe returns
`Probe { ok, checks }`, which responds 200 or 503 with JSON.

| Probe | OK when | Failing it means |
|---|---|---|
| liveness | not wedged | restart the container |
| readiness | started, not shutting down, every dependency up | no traffic for now |
| startup | started | not finished starting: don't run the other probes yet |

**Question**: Your database goes down for two minutes. What happens if the
liveness probe checks the database?

---

## Exercise 3: Graceful Shutdown

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Shut down without dropping requests
- Understand drain delays and grace periods

**Task**: `server.rs`

```rust
pub fn app(state: AppState) -> Router;   // /, /slow?ms=, /healthz, /readyz, /startupz, /version
pub async fn serve(listener: TcpListener, state: AppState,
                   shutdown: impl Future<Output = ()> + Send + 'static) -> io::Result<ShutdownReport>;
```

When `shutdown` completes:
1. readiness starts failing (`begin_shutdown`)
2. keep serving for `drain_delay` — load balancers only notice on their next check
3. stop accepting; let in-flight requests finish (axum's `with_graceful_shutdown`)
4. if they take longer than `shutdown_timeout`, give up: `drained_cleanly: false`

`shutdown_signal()` (provided) waits for Ctrl-C or SIGTERM.

**Questions**: Kubernetes waits 30 s (`terminationGracePeriodSeconds`) between
SIGTERM and SIGKILL. How should `DRAIN_DELAY_MS` and `SHUTDOWN_TIMEOUT_MS`
relate to it? And why does `ENTRYPOINT app serve` (no brackets) break all of
this?

---

## Exercise 4: A Health Check Without curl

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Speak minimal HTTP/1.1 over TCP
- Put a time limit on a whole operation

`scratch` and distroless images have no shell and no curl, so
`HEALTHCHECK CMD curl ...` can't work. The binary checks itself instead:
`HEALTHCHECK CMD ["/app", "healthcheck"]`.

```rust
pub async fn probe(addr: SocketAddr, path: &str, timeout: Duration) -> Result<u16, ProbeError>;
pub fn exit_code(result: &Result<u16, ProbeError>) -> i32;   // Docker: 0 healthy, 1 unhealthy
```

Send `GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n`,
parse the status line. Errors: `Connect`, `Timeout` (connected but no
answer — the whole exchange gets one deadline), `BadResponse` (not HTTP),
`Unhealthy(status)` (not 2xx).

---

## Exercise 5: Build Metadata

**Difficulty**: Easy
**Time**: 25 minutes

`build.rs` (provided — read it) sets `BUILD_GIT_SHA`, `BUILD_PROFILE` and
`BUILD_TARGET` at compile time. Implement `build_info::current()` with
`env!`, and `linkage()`: `"static"` when `cfg!(target_feature = "crt-static")`.
`GET /version` returns it as JSON.

**Question**: inside `docker build` there's usually no `.git` directory. How
does the commit get into the binary?

---

## Exercise 6: A Dockerfile Linter

**Difficulty**: Hard
**Time**: 60 minutes

**Learning Objectives**:
- Know why each Dockerfile best practice exists
- Parse a line-based format with continuations

```rust
pub fn parse(text: &str) -> Vec<Instruction>;   // skip comments/blank lines, join `\` continuations
pub fn lint(text: &str) -> Vec<Finding>;         // sorted by (line, rule)
```

| Rule | Fires when |
|---|---|
| `UnpinnedBaseImage` | `FROM` without a tag, or `:latest` (not `scratch`, digests, or earlier stages) |
| `SingleStage` | only one `FROM` |
| `RunsAsRoot` | the final stage has no `USER`, or `USER root` / `0` |
| `NoHealthcheck` | the final stage has no `HEALTHCHECK` |
| `NoDependencyCaching` | `RUN cargo build` without `--mount=type=cache`, in a stage without `cargo chef cook` |
| `AddForLocalFiles` | `ADD` of anything but an http(s) URL |
| `ShellFormCommand` | `CMD`/`ENTRYPOINT` not in JSON-array form |

The phase README's example Dockerfile produces three findings — which?

---

## Exercise 7: Compose Start Order

**Difficulty**: Medium
**Time**: 45 minutes

```rust
pub fn parse(yaml: &str) -> Result<ComposeFile, ComposeError>;
pub fn startup_order(file: &ComposeFile) -> Result<Vec<Vec<String>>, ComposeError>;
```

- `depends_on` comes in two syntaxes: a list, or a map with `condition`
- return *waves*: services whose dependencies are all in earlier waves; sorted
- errors: an unknown dependency; `service_healthy` on a service without a
  `healthcheck`; a cycle (list every service that can't start)

---

## Exercise 8: The Images and the Stack

**Difficulty**: Medium
**Time**: 60 minutes (plus build time)

Rewrite `exercise/docker/debian.Dockerfile` and `scratch.Dockerfile` (they
start as the phase README's example) until the linter has nothing to say,
then build them:

- **debian**: `rust:<version>-slim-trixie` build stage → `debian:trixie-slim`
- **scratch**: `rust:<version>-alpine` (musl, static) → `FROM scratch`, under 10 MB
- both: BuildKit cache mounts, `USER 10001:10001`,
  `HEALTHCHECK CMD ["/app", "healthcheck"]`, exec-form `ENTRYPOINT`

`exercise/docker/compose.yaml` runs Postgres, the API (after the database is
healthy) and a second instance (after the API is healthy). The tests check
your Dockerfiles with your linter, and the Compose file with your Exercise 7.

```bash
docker build -f 05-cloud-native/21-containerization/exercise/docker/scratch.Dockerfile -t rustlearn/m21-mine:scratch .
docker images rustlearn/m21-mine                 # size?
docker compose -f 05-cloud-native/21-containerization/exercise/docker/compose.yaml up --build
```

**Question**: with `docker stop`, how long until the container exits, and
why?

---

## Bonus Challenge: Readiness That Follows a Dependency

**Difficulty**: Medium
**Time**: 40 minutes

`dep_watch.rs`:
- `Hysteresis::new(failures, successes)` — change state only after N
  consecutive failures / M consecutive successes; starts down
- `host_port("postgres://u:p@db:5432/app") == Some("db:5432")`
- `watch_tcp(name, addr, health, config)` — try a TCP connection every
  `interval`, and record the smoothed result as a dependency

The `serve` command starts a watcher when `DATABASE_URL` is set, so in the
Compose stack `docker compose stop db` makes the API's `/readyz` fail.

---

## Check Your Understanding

- [ ] Explain liveness vs readiness vs startup, and what goes in each
- [ ] Shut a service down without losing requests
- [ ] Explain why the exec form of `ENTRYPOINT` matters
- [ ] Build a multi-stage image with dependency caching
- [ ] Build a static binary and ship it `FROM scratch`
- [ ] Pass configuration and secrets into a container
- [ ] Order services with Compose health checks

---

## Additional Resources

- [The Twelve-Factor App](https://12factor.net/)
- [Docker: Rust language guide](https://docs.docker.com/guides/rust/)
- [Dockerfile reference](https://docs.docker.com/reference/dockerfile/) and [build cache](https://docs.docker.com/build/cache/)
- [cargo-chef](https://github.com/LukeMathWalker/cargo-chef)
- [hadolint](https://github.com/hadolint/hadolint) — the real Dockerfile linter
- [Kubernetes: liveness, readiness and startup probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
- [Compose: control startup order](https://docs.docker.com/compose/how-tos/startup-order/)
