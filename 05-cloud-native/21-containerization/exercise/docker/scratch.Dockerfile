# syntax=docker/dockerfile:1
#
# Exercise 8: turn this into a good scratch (static musl) image.
# The tests lint this file: cargo test -p m21-containerization-tests --features mine ex8_
#
# Build from the REPOSITORY ROOT (the crate is part of the Cargo workspace):
#   docker build -f 05-cloud-native/21-containerization/exercise/docker/scratch.Dockerfile -t rustlearn/m21-mine:scratch .
#
# This starting point is the example from the phase README. Problems to fix:
# no dependency caching, root user, no health check -- see exercises.md.

FROM rust:1.94 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p m21-containerization

FROM debian:trixie-slim
COPY --from=builder /app/target/release/m21-containerization /usr/local/bin/app
CMD ["app", "serve"]
