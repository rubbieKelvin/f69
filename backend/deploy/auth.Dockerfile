# syntax=docker/dockerfile:1
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p auth-service

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -r -u 65532 -s /sbin/nologin app
COPY --from=builder /app/target/release/auth-service /usr/local/bin/auth-service
USER app
ENTRYPOINT ["/usr/local/bin/auth-service"]
