# Answers · 21 Containerization

## Exercise 1: Why `0.0.0.0` and not `127.0.0.1`?

A container has its own network namespace, with its own loopback interface.
`127.0.0.1` inside the container is reachable only from inside it; port
publishing (`-p 8080:8080`) and other containers connect through the
container's network interface. Binding `0.0.0.0` listens on all of them.

## Exercise 2: The database is down for two minutes and liveness checks it

Every replica fails liveness at the same time, so the orchestrator restarts
all of them — repeatedly, with growing back-off (`CrashLoopBackOff`). When the
database comes back, the service is still restarting, cold, and every replica
reconnects at once. A failing *readiness* probe would have just stopped
traffic, and the replicas would have resumed the moment the database
recovered. Liveness answers "would a restart fix this?", and for a dependency
outage the answer is no.

## Exercise 3

**How should the drain delay and shutdown timeout relate to the grace
period?** `drain delay + shutdown timeout` must be less than the grace period
(Kubernetes: `terminationGracePeriodSeconds`, 30 s by default; Docker's
`docker stop`: 10 s, `--time` or Compose's `stop_grace_period` to change it),
with a margin. Otherwise SIGKILL arrives mid-drain. The defaults here are
5 s + 20 s < 30 s; the Compose file sets `stop_grace_period: 30s` to match.

The drain delay exists because removing a pod from a load balancer isn't
instant: Kubernetes updates endpoints, kube-proxy/ingress controllers reload,
and in the meantime new requests still arrive. Kubernetes runs a `preStop`
hook before sending SIGTERM for the same reason; some teams put the sleep
there instead.

**Why does `ENTRYPOINT app serve` break it?** The shell form runs
`/bin/sh -c "app serve"`, so `sh` is PID 1 and the app a child. `docker stop`
sends SIGTERM to PID 1; `sh` doesn't forward it, so the app never starts
draining, and is SIGKILLed when the grace period runs out. (In `scratch`
there's no `/bin/sh` at all, so the container doesn't even start.) PID 1 also
gets special signal treatment: signals without a handler are ignored, which
is why the app installs its own SIGTERM handler.

## Exercise 5: How does the commit get in without `.git`?

The Dockerfiles declare `ARG GIT_SHA`, so `docker build --build-arg
GIT_SHA=$(git rev-parse --short=12 HEAD)` makes it an environment variable
for the following `RUN` steps. `build.rs` prefers `GIT_SHA` over running
`git`, and `cargo:rerun-if-env-changed=GIT_SHA` makes Cargo re-run it when the
value changes. (The `.dockerignore` excludes `.git`, which is large and
changes with every commit, busting the cache.)

## Exercise 6: The phase README's example

```
line 4: NoDependencyCaching   RUN cargo build --release, no cache mount, no cargo-chef
line 6: RunsAsRoot            final stage has no USER
line 6: NoHealthcheck         final stage has no HEALTHCHECK
```

The base images are pinned (`rust:1.75`, `debian:bookworm-slim`), it's
multi-stage, and `CMD ["app"]` is in exec form — so those rules pass.

## Exercise 8: How long does `docker stop` take?

About the drain delay plus however long in-flight requests need: the app
receives SIGTERM (exec form), fails readiness, waits `DRAIN_DELAY_MS`, then
finishes in-flight requests and exits. With nothing in flight and the Compose
file's `DRAIN_DELAY_MS=2000`, about 2 s. If the app *ignored* SIGTERM, it
would be the full grace period (10 s by default) and then SIGKILL.

Expected image sizes: the scratch image is the binary alone (around 1–2 MB
with the size-focused profile); the Debian one adds ~75–80 MB of base image.
