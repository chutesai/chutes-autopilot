# Implementation Plan

## Goal

Expand AutoPilot routing to support all chat-capable Chutes LLM models (TEE and non-TEE) while preserving the existing utilization-based ranking, stickiness, and streaming-safe failover semantics.

Specifically:
- AutoPilot alias (`chutesai/AutoPilot`) should consider both TEE (`confidential_compute=true`) and non-TEE models.
- AutoPilot must not select non-chat chutes that appear in the utilization feed (images, embeddings, etc). Eligibility must be constrained by an authoritative chat-model catalog.

## Scope

This plan addresses human queue item **2026-02-13 19:35:53**:
- "update the entire repo to allow non-TEE models ... allow all LLMs that meet the criteria with or without TEE and the rest of the filtering and ranking"

Out of scope (future):
- Additional OpenAI endpoints beyond `POST /v1/chat/completions`
- Cryptographic attestation verification (beyond metadata)

## Definition of Done

- Control plane:
  - Maintain an in-memory allowlist of eligible chat-capable model ids from `GET https://llm.chutes.ai/v1/models` (configurable).
  - Candidate ranking filters utilization entries to that allowlist when available, and never routes to utilization-only models not present in the catalog.
  - Utilization ranking formula and deterministic ordering are unchanged.
- Data plane:
  - `chutesai/AutoPilot` routes to the top-ranked eligible model (TEE or non-TEE) and sets `x-chutes-autopilot-selected`.
  - Comma-separated `model` lists may include non-TEE models; items are validated against the model catalog when available.
  - Direct single-model requests may target non-TEE models; validated against the model catalog when available.
  - Locally-generated `4xx/5xx` errors use OpenAI `ErrorResponse` shape.
- Quality gates:
  - `cargo test`
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
- Docs/config:
  - README and `.env.example` reflect non-TEE support and the model catalog filter.
  - `docker-compose.yaml` exposes any new env vars.
  - `.gitignore` continues to ignore local secrets/runtime output.

## Decisions / Assumptions

- The authoritative definition of "eligible LLM chat models" is the upstream OpenAI-style model catalog (`/v1/models`) served by the same backend we proxy.
- The utilization feed is a broad "chute inventory" signal and must be constrained by the model catalog.
- TEE metadata (`tee` / `confidential_compute`) is not enforced by default. (Optional future knob.)

## Implementation Steps

1. Specs and research alignment
- [ ] Update specs:
  - [ ] `specs/003-utilization-and-ranking/spec.md`: replace TEE allowlist with model-catalog allowlist.
  - [ ] `specs/002-autopilot-mvp/spec.md`: allow non-TEE direct models; validate against model catalog when available.
  - [ ] `specs/004-user-model-preference-list/spec.md`: allow non-TEE lists; validate against model catalog when available.
- [ ] Update research artifacts and coverage matrix to reflect new policy (non-TEE + model catalog).

2. Configuration surface
- [ ] Add env vars:
  - [ ] `MODELS_URL` (default `https://llm.chutes.ai/v1/models`)
  - [ ] `MODELS_REFRESH_MS` (default `300000`)
- [ ] Deprecate/remove TEE-only config/env (`CHUTES_LIST_URL`, `CHUTES_LIST_REFRESH_MS`) if no longer used, or keep for a transition period with clear naming.

3. Control plane: model allowlist refresh
- [ ] Implement a background refresh task:
  - [ ] Fetch `MODELS_URL`, parse `data[].id` into `HashSet<String>`.
  - [ ] Keep last-known-good allowlist on refresh failure.
  - [ ] Store allowlist + timestamp in runtime state for visibility/debug.

4. Control plane: candidate ranking update
- [ ] Replace TEE allowlist usage with model allowlist usage:
  - [ ] When allowlist is non-empty, require utilization record `name` is in allowlist.
  - [ ] When allowlist is empty, use conservative fallback eligibility (recommended: `name` ends with `-TEE`).
- [ ] Update ranking tests and fixtures to include non-TEE candidates and ensure non-model-catalog candidates are excluded.

5. Data plane: request validation update
- [ ] Remove TEE-only validation for direct and explicit list modes.
- [ ] When model allowlist is non-empty:
  - [ ] Reject unknown model ids with `400` (`error.param="model"` and a stable code like `unknown_model`).
- [ ] When model allowlist is empty:
  - [ ] Do not pre-validate direct/list items (proxy upstream and let upstream enforce).

6. Tests
- [ ] Update tests that expect `model_not_tee` to the new codes/behavior.
- [ ] Add/adjust tests for:
  - [ ] Utilization contains a non-catalog model: excluded from ranked candidates.
  - [ ] Explicit model list with an unknown item: `400` when allowlist is available.
  - [ ] Explicit model list contains non-TEE but catalog-valid items: allowed.

7. Docs + config files
- [ ] Update `README.md`, `.env.example`, `docker-compose.yaml` to reflect:
  - non-TEE support
  - model catalog allowlist filtering
  - any new env vars

8. Verification
- [ ] Run `make test` and `make lint`.

## Notes on Current State

- The existing implementation is TEE-only and uses the chute catalog `tee` field as the eligibility source. It will need refactoring per steps above.

