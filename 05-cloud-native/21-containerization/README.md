# 21 · Containerization

## Overview

A container image packages a program with everything it needs, and an
orchestrator (Docker, Compose, Kubernetes) runs it. Rust suits containers
well: one binary, no runtime, and with musl, no shared libraries either —
the whole image can be a few megabytes. But a good container is more than a
small image. The process inside has to cooperate with whatever runs it:
take its configuration from the environment, answer health probes, and shut
down cleanly when asked.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Configuration | Twelve-factor config, every error at once, secrets from files |
| 2 | Probes | Liveness vs readiness vs startup |
| 3 | Graceful shutdown | Drain, finish in-flight requests, exit within the grace period |
| 4 | Built-in health check | No curl in `scratch`: the binary probes itself |
| 5 | Build metadata | Commit and version baked in with `build.rs` |
| 6 | Dockerfile linter | The best practices, as code |
| 7 | Compose ordering | `depends_on` conditions as a topological sort |
| 8 | Images and stack | Multi-stage builds, cache mounts, static musl, Compose |
| Bonus | Dependency watching | Readiness that follows the database, with hysteresis |

## Key Concepts

### Multi-stage builds

```dockerfile
FROM rust:1.94-alpine AS build        # the compiler: ~1 GB, never shipped
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release ...

FROM scratch                           # what ships: just the binary
COPY --from=build /app /app
```

Only the last stage becomes the image. Cache mounts (or
[cargo-chef](https://github.com/LukeMathWalker/cargo-chef)) keep compiled
dependencies between builds, so changing one line of source doesn't
recompile 300 crates.

### Base images

| Final stage | Size | Has a shell? | Needs |
|---|---|---|---|
| `debian:trixie-slim` | ~80 MB + app | yes | a glibc binary (the default target) |
| `gcr.io/distroless/cc` | ~25 MB + app | no | glibc binary; includes CA certificates |
| `alpine` | ~8 MB + app | yes | a musl binary |
| `scratch` | the app only | no | a **static** binary (musl), plus CA certificates if it makes TLS calls |

### Probes

```
startup   ──passes once──▶ liveness and readiness start running
liveness  fails ─▶ restart the container         (process is stuck)
readiness fails ─▶ remove from the load balancer (can't serve right now)
```

### Graceful shutdown

```
SIGTERM ─▶ readiness fails ─▶ drain delay (still serving) ─▶ stop accepting
        ─▶ in-flight requests finish (≤ timeout) ─▶ exit
                                              grace period ends ─▶ SIGKILL
```

The process only gets SIGTERM if it *is* PID 1: use the exec form,
`ENTRYPOINT ["/app"]`. In shell form, `/bin/sh` is PID 1 and doesn't forward
signals; after the grace period everything is killed mid-request.

## Common Pitfalls

1. **Binding to `127.0.0.1`** — unreachable from outside the container; bind `0.0.0.0`
2. **`COPY . .` then `cargo build`** with no caching — every build compiles every dependency
3. **Running as root** — a container escape starts with root privileges
4. **Checking dependencies in liveness** — a database blip restarts every replica
5. **Shell-form `CMD`/`ENTRYPOINT`** — SIGTERM is lost; every deploy drops requests
6. **Secrets in `ENV` or build args** — they end up in the image history (`docker history`)
7. **`FROM rust:latest`** — builds change under you; pin versions
8. **`depends_on` without a condition** — "started" isn't "ready"; use `service_healthy`

## Running This Module

```bash
cargo run  -p m21-containerization -- demo                  # your code
cargo test -p m21-containerization-tests --features mine    # test your code
cargo run  -p m21-containerization-solution -- demo         # the solution, all parts
cargo test -p m21-containerization-tests                    # 20 tests against the solution
```

With Docker (optional), from the repository root:

```bash
docker build -f 05-cloud-native/21-containerization/solution/docker/scratch.Dockerfile -t rustlearn/m21:scratch .
docker run --rm -p 8080:8080 rustlearn/m21:scratch
docker compose -f 05-cloud-native/21-containerization/solution/docker/compose.yaml up --build
```

See [GETTING_STARTED.md](GETTING_STARTED.md) for a full walkthrough.

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the four exercises in
[the phase README](../README.md) (a minimal image under 10 MB, a multi-stage
pipeline, Compose for several services, health checks).

- **Everything is tested without Docker.** The tests run the service, its
  probes and its shutdown in-process, and check the Dockerfiles and Compose
  file with the code from Exercises 6 and 7. Building and running the images
  is a documented, optional step.
- **The Docker files haven't been built in this repository's own checks**
  (Docker wasn't running there). They pass the module's linter and Compose
  checks; if a `docker build` fails for you, please open an issue.
- **The phase README's example Dockerfile** (`FROM rust:1.75 as builder ...`)
  works, but rebuilds every dependency on each change and runs as root without
  a health check — Exercise 6's linter finds exactly those three problems.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[22 · Kubernetes Operators](../22-kubernetes-operators/)
