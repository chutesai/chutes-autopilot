# Multi-stage build for a small, reproducible, non-root runtime image.
ARG RUST_VERSION=1.93.1
ARG APP_NAME=chutes-autopilot
ARG APP_UID=10001
ARG APP_GID=10001
ARG APP_USER=autopilot

FROM rust:${RUST_VERSION}-slim AS builder

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
WORKDIR /app

# Install minimal build tooling.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Pre-fetch dependencies to leverage Docker layer caching.
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

# Build the binary.
COPY src ./src
RUN cargo build --release --locked --bin ${APP_NAME} \
    && strip target/release/${APP_NAME}

FROM debian:bookworm-slim AS runtime

ARG APP_UID
ARG APP_GID
ARG APP_USER

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid ${APP_GID} ${APP_USER} \
    && useradd --system --no-create-home --home /app --uid ${APP_UID} --gid ${APP_GID} --shell /usr/sbin/nologin ${APP_USER} \
    && mkdir -p /app \
    && chown ${APP_UID}:${APP_GID} /app

WORKDIR /app
COPY --from=builder --chown=${APP_UID}:${APP_GID} /app/target/release/${APP_NAME} /usr/local/bin/${APP_NAME}

USER ${APP_UID}:${APP_GID}

EXPOSE 8080
ENV LISTEN_ADDR=0.0.0.0:8080
ENV RUST_LOG=info

HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD curl -f http://127.0.0.1:8080/readyz || exit 1

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/chutes-autopilot"]
