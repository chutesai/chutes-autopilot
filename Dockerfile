# Multi-stage build for smaller runtime images.
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/chutes-autopilot /app/chutes-autopilot

EXPOSE 8080

ENV RUST_LOG=info
ENV LISTEN_ADDR=0.0.0.0:8080

CMD ["/app/chutes-autopilot"]

