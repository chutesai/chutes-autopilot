# Research Summary

<confidence>82</confidence>

## What We Verified (Executable Evidence)

### Utilization Feed Exists, But It Contains Non-Chat Chutes

The endpoint `https://api.chutes.ai/chutes/utilization` returns HTTP 200 with a JSON array payload.

Observed (sampled 2026-02-14):
- Array length: `537` entries
- Many entries have `name == "[private chute]"` (private entries are present and must be filtered).
- Count with `name == "[private chute]"`: `435`
- Count with `name != "[private chute]"` and `active_instance_count > 0`: `100`
- Key fields required by the ranking algorithm are present in the sample:
  - `name`
  - `active_instance_count`
  - `utilization_current`, `utilization_5m`, `utilization_15m`, `utilization_1h`
  - `rate_limit_ratio_5m`, `rate_limit_ratio_15m`, `rate_limit_ratio_1h`
  - `scalable`, `scale_allowance`

Important finding:
- Not all public utilization entries correspond to LLM chat models. Examples present in utilization but absent from the OpenAI-style model catalog include image and embedding models such as:
  - `FLUX.1-schnell`
  - `Qwen-Image-2512`
  - `BAAI/bge-m3`

This means AutoPilot candidate ranking must be filtered by an LLM model catalog, not just by utilization alone, otherwise the router can select a non-chat chute for `POST /v1/chat/completions`.

### Upstream Model Catalog Exists and Includes TEE + Non-TEE Models

`https://llm.chutes.ai/v1/models` returns HTTP 200 with an OpenAI-style model list (`{"object":"list","data":[...]}`) and per-model metadata including `confidential_compute`.

Observed (sampled 2026-02-14):
- Total models returned: `66`
- `confidential_compute=true`: `22`
- `confidential_compute=false`: `44`

Cross-checks (sampled 2026-02-14):
- All `/v1/models` ids exist in the public Chutes chute catalog (`GET https://api.chutes.ai/chutes/?limit=1000` with `public==true`).
- Of the `100` active public utilization entries, `65` are present in `/v1/models`.
- The remaining `35` active public utilization entries are **not** present in `/v1/models` and should be excluded from AutoPilot selection for chat completions.

### TEE Metadata Still Exists (Optional Policy)

Chutes exposes:
- `tee` boolean in the chute catalog (`GET https://api.chutes.ai/chutes/?limit=1000`)
- `confidential_compute` boolean in `/v1/models`

These can support an optional future policy knob (prefer/require confidential compute), but the current request is to allow both TEE and non-TEE LLMs.

### OpenAI Error Response Schema Is Public and Stable

The OpenAI documented OpenAPI spec defines an `ErrorResponse` object with an `error` field, and an `Error` schema with required fields:
- `type` (string)
- `message` (string)
- `param` (string or null)
- `code` (string or null)

Reference: `https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml`

## What Remains Unverified (Needs Confirmation or Runtime Testing)

- Backend `POST /v1/chat/completions` behavior and auth requirements (example given in docs: `https://llm.chutes.ai`).
- Exact OpenAI-compat behavior expected beyond `POST /v1/chat/completions`.
- Semantics for user-specified model preference lists inside `model` (comma-separated list) need integration testing for latency/error behavior, especially when items include both TEE and non-TEE models.
- Operational behavior when the eligible candidate set is empty (readiness + error response conventions), including startup windows where the model catalog allowlist has not been fetched yet.
- Whether the `tee` and/or `confidential_compute` signals imply a cryptographic attestation guarantee, and how (if at all) that guarantee should be validated (e.g., via `/chutes/{id_or_name}/evidence`).
- Stickiness keying is specified (auth token hash preferred, otherwise requester IP). What still needs verification is deployment-specific proxy behavior:
  - whether the service will sit behind a reverse proxy that overwrites `X-Forwarded-For`
  - which proxy IP ranges should be treated as trusted for `X-Forwarded-For` stickiness
- Whether any clients depend on strict JSON request formatting (key order, float formatting). We should assume "no" and test with representative SDKs.

## Recommended Design Update (Non-TEE Support)

- Use `GET https://llm.chutes.ai/v1/models` as the authoritative allowlist for chat-capable LLM models.
- Filter utilization-based candidates to those present in that allowlist (when the allowlist is available and non-empty) to avoid routing to image/embedding chutes.
- Allow both TEE and non-TEE models; treat TEE metadata as optional policy only.

## Immediate Next Steps

- Update specs + implementation plan to remove TEE-only enforcement and introduce the model-catalog allowlist.
- Plan the code changes to:
  - refresh `/v1/models` into an in-memory allowlist
  - update candidate ranking and request validation to use that allowlist
  - update tests and fixtures accordingly
