# Dependency Research

This project is intentionally minimal-dependency and uses primary-source docs where possible.

## Rust (Selected Stack)

### Async Runtime

- `tokio` docs: `https://docs.rs/tokio`

### HTTP Server

- `axum` docs: `https://docs.rs/axum`
- `hyper` docs: `https://docs.rs/hyper`
- `hyper` guide (bodies and streaming): `https://hyper.rs/guides/1/body/streaming/`
- `tower` docs: `https://docs.rs/tower`
- `tower-http` docs: `https://docs.rs/tower-http`

### HTTP Client

- `reqwest` docs: `https://docs.rs/reqwest`
  - Streaming support via `bytes_stream()`; configure timeouts and avoid accidental decompression when strict passthrough matters.
  - Compression behavior:
    - `ClientBuilder::gzip`, `ClientBuilder::no_gzip`: `https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html`
    - `ClientBuilder::brotli`, `ClientBuilder::no_brotli`: `https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html`
    - `ClientBuilder::zstd`, `ClientBuilder::no_zstd`: `https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html`
    - For strict byte-for-byte passthrough, default to disabling automatic decompression and explicitly manage `Accept-Encoding` and hop-by-hop headers.

### Timeouts

- `tokio::time::timeout` docs: `https://docs.rs/tokio/latest/tokio/time/fn.timeout.html`
  - Use separate timeouts for connect, header, and first-body-byte phases to support streaming while still allowing pre-commit failover.

### JSON

- `serde` docs: `https://docs.rs/serde`
- `serde_json` docs: `https://docs.rs/serde_json`

### Logging

- `tracing` docs: `https://docs.rs/tracing`
- `tracing-subscriber` docs: `https://docs.rs/tracing-subscriber`

### Stickiness (Hashing + Bounded Cache)

- `sha2` docs: `https://docs.rs/sha2`
- `moka` docs: `https://docs.rs/moka` (optional; TTL + max-capacity cache)
- `ipnet` docs: `https://docs.rs/ipnet` (optional; parse `TRUSTED_PROXY_CIDRS`)

### Testing

- `tokio::test` for async tests: `https://docs.rs/tokio/latest/tokio/attr.test.html`

## Go (Fallback Stack)

### HTTP Server and Client

- `net/http` package docs: `https://pkg.go.dev/net/http`
  - `http.Server` timeouts, `http.Transport` reuse guidance, and concurrency notes.
- `net/http#Transport` docs: `https://pkg.go.dev/net/http#Transport`
  - `Transport` should be reused and is safe for concurrent use.
  - `DisableCompression` controls automatic gzip request/transparent decompression behavior (important for strict passthrough proxies).

### HTTP Proxy Semantics (Hop-by-Hop Headers, Connection Reuse)

- RFC 9110 (HTTP Semantics): `https://www.rfc-editor.org/rfc/rfc9110.html`
- RFC 9112 (HTTP/1.1): `https://www.rfc-editor.org/rfc/rfc9112.html`
- RFC 7239 (Forwarded HTTP Extension): `https://www.rfc-editor.org/rfc/rfc7239.html`

### Reverse Proxy Utilities (Optional Reference)

- `net/http/httputil` package docs: `https://pkg.go.dev/net/http/httputil`
  - `ReverseProxy` and `FlushInterval` are relevant for streaming proxies.

### JSON Parsing

- `encoding/json` package docs: `https://pkg.go.dev/encoding/json`
  - Use `json.RawMessage` to preserve unknown fields and minimize mutation surface.

### Logging

- `log/slog` package docs: `https://pkg.go.dev/log/slog`
  - Structured logging; avoid writing sensitive request bodies and auth headers.

### Testing

- `testing` and `net/http/httptest` docs:
  - `https://pkg.go.dev/testing`
  - `https://pkg.go.dev/net/http/httptest`

## OpenAI Compatibility

- OpenAI OpenAPI spec (documented): `https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml`
- OpenAI OpenAPI repo (manual snapshot): `https://github.com/openai/openai-openapi/tree/manual_spec`
- OpenAI Python SDK (Chat Completions resource; shows `model` is a string-like parameter): `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/resources/chat/completions/completions.py`
  - This project targets `POST /v1/chat/completions` compatibility and must preserve upstream status codes and streaming behavior.
  - The comma-separated `model` preference list is an Autopilot extension; Autopilot must rewrite `model` to a single upstream model name per attempt.

## Chutes Control Plane

- Chutes OpenAPI spec (public): `https://api.chutes.ai/openapi.json`
- Chute catalog (public, includes `tee` boolean): `https://api.chutes.ai/chutes/?limit=1000`
- Utilization feed (public): `https://api.chutes.ai/chutes/utilization`
- OpenAI-style model catalog (public): `https://llm.chutes.ai/v1/models`
  - Recommended as the authoritative allowlist for chat-capable LLM models to prevent selecting non-chat chutes present in the utilization feed.
- Evidence endpoint (for deeper TEE validation research): `https://api.chutes.ai/chutes/{chute_id_or_name}/evidence`
- Chutes docs (may require a browser for full navigation): `https://docs.chutes.ai`

## Chutes Data Plane (OpenAI-Compatible Upstream)

- Models endpoint (public): `https://llm.chutes.ai/v1/models`
  - Contains `confidential_compute` (appears to align with `*-TEE` models).
  - Used for:
    - request validation (catch typos/unknown models)
    - utilization filtering (exclude image/embedding chutes from AutoPilot selection)

Note: Streaming proxy correctness (including retry boundaries) should be validated with integration tests regardless of language.
