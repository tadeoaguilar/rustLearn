# syntax=docker/dockerfile:1
#
# A static musl binary in an empty image: a few MB, nothing to attack but the
# app itself -- and nothing to debug with either (no shell, no ls).
# Build from the REPOSITORY ROOT:
#
#   docker build -f 05-cloud-native/21-containerization/solution/docker/scratch.Dockerfile \
#       --build-arg GIT_SHA=$(git rev-parse --short=12 HEAD) -t rustlearn/m21:scratch .

ARG RUST_VERSION=1.94

FROM rust:${RUST_VERSION}-alpine AS build
# Alpine's Rust targets musl, which links statically by default.
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
ARG GIT_SHA=unknown
# Size-focused release settings, set through the environment so the
# workspace's own profile stays untouched.
ENV CARGO_PROFILE_RELEASE_OPT_LEVEL=s \
    CARGO_PROFILE_RELEASE_LTO=true \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_PANIC=abort \
    CARGO_PROFILE_RELEASE_STRIP=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p m21-containerization-solution \
 && cp target/release/m21-containerization-solution /app

FROM scratch
COPY --from=build /app /app
USER 10001:10001
ENV PORT=8080
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app", "healthcheck"]
ENTRYPOINT ["/app"]
CMD ["serve"]
