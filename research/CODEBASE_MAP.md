# Codebase Map

## Repository Structure (Current)

- `Cargo.toml`, `Cargo.lock`
  - Rust crate definition and dependency lockfile.
- `src/main.rs`
  - Process entrypoint: initializes tracing, loads config from env, binds `LISTEN_ADDR`, starts the Axum server.
- `src/lib.rs`
  - Core service implementation: routes, state, control-plane refresh tasks, ranking, request parsing/routing, proxy + failover, and tests.
- `testdata/`
  - Committed fixtures used by tests (e.g. utilization ranking fixture).
- `Makefile`
  - Convenience targets: `make run`, `make test`, `make lint`.
- `README.md`
  - Operational overview and configuration.
- `HUMAN_INSTRUCTIONS.md`
  - Human priority queue inputs (planning only; not runtime behavior).
- `.gitignore`
  - Guardrails for local/secrets/runtime artifacts.
- `logs/`, `consensus/`, `completion_log/`, `.ralphie/`
  - Runtime artifacts produced by the orchestration script (not part of the router service).
- `specs/`
  - Product specs:
    - `specs/001-project-foundation/spec.md`
    - `specs/002-autopilot-mvp/spec.md`
    - `specs/003-utilization-and-ranking/spec.md`
    - `specs/004-user-model-preference-list/spec.md`
    - `specs/005-sticky-selection-and-rotation/spec.md`
- `research/`
  - Research and planning artifacts (this folder).
- `ralphie.sh`, `PROMPT_*.md`, `.specify/memory/constitution.md`
  - Agent automation scaffolding and project constitution (not part of runtime service).

## Runtime Service (Current Behavior)

### External HTTP API (Inbound)

- `POST /v1/chat/completions`
  - OpenAI-compatible chat completions endpoint.
  - Behavior:
    - If request `model` matches an AutoPilot alias, select a chute from the in-memory ranked snapshot and proxy upstream with streaming passthrough.
    - If request `model` contains a comma-separated ordered list of models, attempt them in order (failover list).
    - If request `model` is a single (direct) model name, proxy upstream without adding the Autopilot-selected debug header.

- `GET /healthz` (liveness)
- `GET /readyz` (readiness: has a non-empty candidate snapshot fresher than `READYZ_MAX_SNAPSHOT_AGE_MS`)

### External HTTP APIs (Outbound)

- Chute catalog (TEE allowlist, control plane; current)
  - `GET https://api.chutes.ai/chutes/?limit=1000`
  - Used to build an in-memory allowlist of `tee==true` and `public==true` chute names.
- Utilization feed (control plane)
  - `GET https://api.chutes.ai/chutes/utilization`
  - Polled on an interval; response is a JSON array of chute utilization objects.
- Backend OpenAI-compatible service (data plane)
  - Base URL configurable (example: `https://llm.chutes.ai`)
  - Target path: `/v1/chat/completions`
- Planned: model catalog (eligible-model allowlist, control plane)
  - `GET https://llm.chutes.ai/v1/models`
  - Used to build an in-memory allowlist of chat-capable model ids (TEE and non-TEE) for utilization filtering and request validation.

Reference (outbound, used for planning/verification only):
- Chutes OpenAPI spec: `https://api.chutes.ai/openapi.json`

### Configuration Surfaces

- Listener:
  - `LISTEN_ADDR` (default: `0.0.0.0:8080`)
- `BACKEND_BASE_URL`
- `CHUTES_LIST_URL`
- `CHUTES_LIST_REFRESH_MS`
- Planned: `MODELS_URL` (default: `https://llm.chutes.ai/v1/models`)
- Planned: `MODELS_REFRESH_MS` (default: `300000`)
- `UTILIZATION_URL`
- `UTILIZATION_REFRESH_MS`
- Current policy: TEE-only is enforced (prefer authoritative TEE allowlist; fallback to `-TEE` suffix)
- Planned policy: allow both TEE and non-TEE LLM models; use an OpenAI-style model catalog allowlist (`GET /v1/models`) to filter utilization candidates and validate user-specified models
- Logging configuration (language-specific; e.g. `LOG_LEVEL` or `RUST_LOG`)
- `MAX_REQUEST_BYTES` (defensive bound)
- `MAX_MODEL_LIST_ITEMS` (defensive bound)
- `STICKY_TTL_SECS` / `STICKY_MAX_ENTRIES` (stickiness controls)
- `TRUST_PROXY_HEADERS` / `TRUSTED_PROXY_CIDRS` (safe opt-in for `X-Forwarded-For` stickiness behind a reverse proxy)
- `UPSTREAM_CONNECT_TIMEOUT_MS` (defensive bound)
- `UPSTREAM_HEADER_TIMEOUT_MS` (defensive bound)
- `UPSTREAM_FIRST_BODY_BYTE_TIMEOUT_MS` (defensive bound; used only before any client bytes are committed)
- `READYZ_MAX_SNAPSHOT_AGE_MS` (readiness freshness threshold)

## Integration Boundaries

### Control Plane

- Chute catalog fetcher / TEE allowlist (current):
  - HTTP client fetches chute catalog JSON.
  - Parser extracts `name`, `tee`, `public`.
  - Builds and refreshes an in-memory allowlist used by utilization filtering and request validation.
- Planned: model catalog fetcher / eligible-model allowlist:
  - HTTP client fetches `GET https://llm.chutes.ai/v1/models`.
  - Parser extracts `data[].id` and stores an allowlist of chat-capable model ids (includes both TEE and non-TEE).
  - Utilization ranking filters to this allowlist when available, to avoid selecting non-chat models present in the utilization feed.
- Utilization fetcher:
  - HTTP client fetches utilization JSON.
  - Parser validates required fields and tolerates missing/extra fields.
- Ranker:
  - Deterministic score and sort.
  - Produces an ordered list of candidates.
- Snapshot store:
  - Atomic swap of candidate list used by request hot path.

### Data Plane

- Request parser / mutator:
  - Reads JSON request body and rewrites `model` only when it matches an alias or a model-list routing mode.
- Candidate selection:
  - Picks `candidates[0]` and uses subsequent entries for failover.
  - Applies stickiness: if a per-client sticky model is eligible, try it first.
- Proxy engine:
  - Sends request to upstream backend.
  - Streams upstream response bytes to the client without buffering.
  - Implements failover only when no response bytes have been sent to the client.
  - Treats upstream `429` as non-retryable (proxy it back; no failover).
  - Treats connection errors, timeouts, and upstream `503` as retryable only before any client bytes are committed.
