# --- Build stage ---
FROM rust:1.95-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY . ./

RUN cargo build --release --bin openresponses-server

# --- Runtime stage ---
FROM fedora:44

COPY --from=builder /app/target/release/openresponses-server /usr/local/bin/openresponses-server

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["openresponses-server"]
