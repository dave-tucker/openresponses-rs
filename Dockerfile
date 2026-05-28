# --- Build stage ---
FROM rust:1.85-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates/ ./crates/

RUN cargo build --release --bin openresponses-server

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/openresponses-server /usr/local/bin/openresponses-server

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["openresponses-server"]
