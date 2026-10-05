# syntax=docker/dockerfile:1
#
# A Debian-based image: easy to debug (there's a shell), ~90 MB.
# Build from the REPOSITORY ROOT -- the crate is part of the Cargo workspace:
#
#   docker build -f 05-cloud-native/21-containerization/solution/docker/debian.Dockerfile \
#       --build-arg GIT_SHA=$(git rev-parse --short=12 HEAD) -t rustlearn/m21:debian .

ARG RUST_VERSION=1.94

# ---- build stage: the compiler lives only here ----
FROM rust:${RUST_VERSION}-slim-trixie AS build
WORKDIR /src
COPY . .
ARG GIT_SHA=unknown
# Cache mounts keep the registry and target/ between builds, so a source
# change recompiles only our crate. (They aren't part of any image layer, so
# the binary is copied out in the same RUN.)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p m21-containerization-solution \
 && cp target/release/m21-containerization-solution /app

# ---- runtime stage: just the binary ----
FROM debian:trixie-slim
COPY --from=build /app /usr/local/bin/app
# Any non-zero UID works; it doesn't need an entry in /etc/passwd.
USER 10001:10001
ENV PORT=8080
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/app", "healthcheck"]
# Exec form: the app is PID 1 and receives SIGTERM itself.
ENTRYPOINT ["/usr/local/bin/app"]
CMD ["serve"]
