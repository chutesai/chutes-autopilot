# Architecture Options

Implementation language is locked to Rust (per human priority queue). Go options remain as a fallback reference if Rust streaming proxy semantics become unexpectedly complex.

## Option A (Fallback): Go `net/http` Service (Stdlib-First)

### Components

- **Control Plane**
  - Background goroutine runs on a ticker (`UTILIZATION_REFRESH_MS`)
  - Periodically fetches an OpenAI-style model catalog (`GET https://llm.chutes.ai/v1/models`) to maintain an allowlist of **chat-capable LLM model ids** (includes both TEE and non-TEE)
  - Fetches `UTILIZATION_URL`
  - Parses JSON and produces an ordered `[]Candidate`
  - Atomically swaps the latest snapshot used by the hot path

- **Data Plane**
  - `POST /v1/chat/completions`
  - Parse request JSON to select routing mode:
    - AutoPilot alias (ranked candidates)
    - user-specified model list (comma-separated `model`, ordered failover)
    - direct passthrough
  - Apply stickiness (per-client preferred model) when multiple candidates exist
  - Try candidates in order (failover only if no client bytes have been sent)
  - Stream upstream response bytes directly to the client

### Pros

- Small, maintainable codebase with few dependencies
- Strong streaming support with straightforward semantics
- Easy operationalization (single static binary, simple container)

### Cons

- Requires careful request-body rewriting to avoid unintended transformations (use `json.RawMessage` to preserve unknown fields)

## Option B: Go + `httputil.ReverseProxy` (Customized)

### Idea

Use `httputil.ReverseProxy` for hop-by-hop header management and streaming, while injecting custom selection logic and request-body rewriting.

### Pros

- Leverages standard reverse-proxy utilities

### Cons

- Still needs custom logic for candidate failover and model rewriting
- ReverseProxy patterns can obscure retry boundaries (need hard guarantees about "no bytes written yet")

## Option C: Rust (axum/hyper)

Status: **Selected**

### Pros

- Excellent performance characteristics
- Strong typing and explicit concurrency

### Cons

- Higher implementation complexity for streaming proxy + failover rules
- More dependencies and a steeper local dev curve if the org is Go-centric

## Decision Criteria

- Correct, transparent streaming behavior under load
- Simple failover semantics (pre-bytes only)
- Minimal request mutation footprint
- Deterministic eligibility filtering (must not select non-chat models from the utilization feed)
- Operational simplicity (timeouts, resource bounds, observability)
