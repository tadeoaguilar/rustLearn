# Getting Started with 21 · Containerization

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m21-containerization-solution -- demo
```

prints a configuration error report, probes a running server, shuts it down
while a request is in flight, lints two Dockerfiles and orders a Compose
stack.

## What Is Already Here

```
21-containerization/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/                 # ← YOUR WORKSPACE (package m21-containerization)
│   ├── build.rs              #   provided: build metadata (Ex 5)
│   ├── src/config.rs         #   Ex 1
│   ├── src/health.rs         #   Ex 2
│   ├── src/server.rs         #   Ex 3 (shutdown_signal provided)
│   ├── src/probe.rs          #   Ex 4
│   ├── src/build_info.rs     #   Ex 5
│   ├── src/dockerfile_lint.rs#   Ex 6
│   ├── src/compose.rs        #   Ex 7
│   ├── src/dep_watch.rs      #   bonus
│   ├── docker/               #   Ex 8: Dockerfiles to fix, compose.yaml, dev secrets
│   └── src/main.rs           #   runner: demo / serve / healthcheck / lint
├── solution/                 # ← REFERENCE (m21-containerization-solution) + ANSWERS.md
│   └── docker/               #   finished Dockerfiles and compose.yaml
└── tests/                    # ← 20 tests (m21-containerization-tests)
```

## The Commands You Need

```bash
cargo run  -p m21-containerization -- demo
cargo test -p m21-containerization-tests --features mine
cargo test -p m21-containerization-tests --features mine ex3_     # one exercise
cargo run  -p m21-containerization -- lint 05-cloud-native/21-containerization/exercise/docker/debian.Dockerfile
cargo test -p m21-containerization-tests                          # the solution: always green
```

## Run the Service Like a Container Would

```bash
PORT=9000 LOG_LEVEL=debug DRAIN_DELAY_MS=3000 cargo run -p m21-containerization-solution -- serve
curl -i localhost:9000/readyz
curl localhost:9000/version
PORT=9000 cargo run -q -p m21-containerization-solution -- healthcheck; echo "exit code $?"
```

Then, in another terminal, start a slow request and press **Ctrl-C** in the
server's terminal while it's running:

```bash
curl 'localhost:9000/slow?ms=5000'      # still answers: the server drains first
```

Try a broken configuration: `PORT=x LOG_LEVEL=y cargo run -p m21-containerization-solution -- serve`.

## With Docker (Optional)

Start Docker Desktop (or any engine with BuildKit), then from the repository
root:

```bash
# Build both images (the first build downloads the Rust image and compiles; later ones reuse the cache)
docker build -f 05-cloud-native/21-containerization/solution/docker/debian.Dockerfile  -t rustlearn/m21:debian  .
docker build -f 05-cloud-native/21-containerization/solution/docker/scratch.Dockerfile -t rustlearn/m21:scratch \
    --build-arg GIT_SHA=$(git rev-parse --short=12 HEAD) .
docker images rustlearn/m21                       # compare the sizes

# Run one; watch the health status change from "starting" to "healthy"
docker run -d --name m21 -p 8080:8080 rustlearn/m21:scratch
docker ps                                         # STATUS column
curl localhost:8080/version
docker stop m21 && docker logs m21                # "shutdown requested: draining"
docker rm m21

# The whole stack
docker compose -f 05-cloud-native/21-containerization/solution/docker/compose.yaml up --build
docker compose -f 05-cloud-native/21-containerization/solution/docker/compose.yaml stop db   # API readiness turns 503 (bonus)
docker compose -f 05-cloud-native/21-containerization/solution/docker/compose.yaml down
```

The secrets in `docker/secrets/` are throwaway development values.

## If You Get Stuck

1. **`env!("BUILD_GIT_SHA")` not defined** — `build.rs` must be next to `Cargo.toml`; check `cargo build -vv` output for it.
2. **Graceful shutdown test hangs** — the drain future must finish *after* the drain delay, and the timeout must start only then; `tokio::select!` the server against "drain finished + timeout".
3. **The hung-server probe test takes forever** — put one `tokio::time::timeout` around connect, write *and* read.
4. **Linter misses `USER`** — only the *final* stage counts, and the *last* `USER` in it.
5. **Compose cycle reports too few services** — report everything left when no service can start.
6. **`docker build` can't find the workspace** — the build context must be the repository root (the trailing `.`), with `-f` pointing at the Dockerfile.
7. Compare with `solution/src/` — same file and function names.
