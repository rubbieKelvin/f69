# syntax=docker/dockerfile:1
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY auth ./auth
COPY flag ./flag
COPY shared ./shared
RUN cargo build --release -p flag-service

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -r -u 65532 -s /sbin/nologin app
COPY --from=builder /app/target/release/flag-service /usr/local/bin/flag-service
USER app
ENTRYPOINT ["/usr/local/bin/flag-service"]
