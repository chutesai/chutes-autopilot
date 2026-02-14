# Risks and Mitigations

## Streaming Retry Boundary Is Easy to Get Wrong

Risk:
- Retrying after partially writing a streaming response corrupts the client experience.

Mitigations:
- Do not write any response headers/body to the client until an upstream attempt is selected.
- Only fail over on connection errors, timeouts before any committed bytes, or upstream `503` when *no bytes have been written*.
- Add tests that confirm:
  - `503` before streaming triggers retry
  - Any body bytes written disables retry

## Retrying on 429 Can Bypass Rate Limits (Abuse Risk)

Risk:
- If Autopilot retries a request across multiple chutes after receiving an upstream `429`, it can multiply load and effectively help a client bypass user-caused rate limiting.

Mitigations:
- Treat upstream `429` as non-retryable and proxy it back to the client (no failover).
- Do not rotate stickiness on `429`.

## Utilization Feed Outages / Bad Data

Risk:
- Empty or stale candidate snapshot causes request failures.

Mitigations:
- Keep last-known-good snapshot if refresh fails.
- Track snapshot age; expose in `/readyz` and logs (gate `/readyz` on `READYZ_MAX_SNAPSHOT_AGE_MS`).
- Define behavior when snapshot is empty: return `503` with a clear error.

## Model Catalog Fetch Failures (Eligibility Allowlist Staleness)

Risk:
- If the OpenAI-style model catalog (`/v1/models`) cannot be fetched (network outage, auth requirement change, upstream downtime), the eligibility allowlist can be empty or stale. This can reduce capacity (falling back to a conservative set) or block strict validation that catches typos.

Mitigations:
- Keep last-known-good allowlist on refresh failure.
- Log allowlist age (and optionally expose in readiness details) so operators can detect staleness.
- Provide `MODELS_URL` as a configuration knob so deployments can point at an accessible model catalog endpoint.

## Timeouts Can Cause False Failovers (Or Hanging Requests)

Risk:
- If timeouts are too aggressive, Autopilot can trigger unnecessary failovers and duplicate work.
- If timeouts are too lax, clients can experience long hangs when a chute is unreachable or stuck before emitting any bytes.

Mitigations:
- Separate timeouts for connect, response headers, and "first body byte" (pre-commit) and make them configurable.
- Only apply failover timeouts before any committed bytes are sent to the client.

## Selecting Non-Chat Models From The Utilization Feed

Risk:
- The utilization feed includes public chutes that are not chat-capable LLMs (for example image or embedding chutes). If AutoPilot ranks/chooses from utilization without filtering against a chat-capable model catalog, `POST /v1/chat/completions` can be routed to an incompatible model and fail.

Mitigations:
- Maintain an allowlist of eligible chat-capable model ids from `GET https://llm.chutes.ai/v1/models` and filter utilization candidates by that set when available.
- When the allowlist is unavailable/empty, degrade to a conservative fallback eligible set (for example `-TEE` suffix heuristic) rather than selecting arbitrary utilization entries.

## Allowing Non-TEE Models Changes Confidentiality Guarantees

Risk:
- Routing to non-TEE models may violate user expectations if they assumed confidential compute was guaranteed.

Mitigations:
- Make the policy explicit in docs: AutoPilot can select both TEE and non-TEE models.
- (Optional) Provide a config knob to require confidential compute (`confidential_compute==true`) for deployments that need it.
- If stronger guarantees are required, investigate `/chutes/{id_or_name}/evidence` and define explicit attestation validation requirements.

## Unbounded Request Body Size

Risk:
- Reading request body to rewrite JSON can enable memory abuse.

Mitigations:
- Enforce `MAX_REQUEST_BYTES` and return `413` on exceed.
- Avoid logging request bodies.

## User-Specified Model Lists Can Amplify Retries and Load

Risk:
- A client can send a very long comma-separated `model` list, causing excessive upstream attempts and increased latency/cost.

Mitigations:
- Enforce `MAX_MODEL_LIST_ITEMS` (return `400` when exceeded).
- De-duplicate model names to avoid repeated attempts.
- (Optional later) Enforce a global per-request max upstream attempts across both AutoPilot and model-list modes.

## User Preference Lists Can Contain Typos or Non-Chat Models

Risk:
- Humans may paste non-existent model names or non-chat model names into the comma-separated `model` list, causing confusing upstream errors (or repeated failovers).

Mitigations:
- Validate preference-list items against the authoritative model catalog (`GET https://llm.chutes.ai/v1/models`) when available (fail fast and return a clear `400` listing invalid items).
- Allow non-TEE models; do not reject items solely on `confidential_compute==false`.

## Accidental Decompression or Header Mutation Breaks "Passthrough"

Risk:
- HTTP client libraries can transparently decompress upstream responses, altering bytes/headers.

Mitigations:
- Configure the upstream transport to avoid implicit decompression when strict passthrough matters.
- Prefer byte-for-byte streaming from upstream `Body` to client `ResponseWriter`.
- Treat hop-by-hop headers carefully (standard proxy behavior).

## Ranking Instability / Chute Flapping

Risk:
- 5s refresh can cause frequent candidate changes.

Mitigations:
- Deterministic sort order with tie-breakers.
- Consider optional hysteresis later (not required for MVP).

## Stickiness Can Leak Sensitive Identifiers or Grow Without Bound

Risk:
- Using `Authorization` tokens or IP addresses directly as cache keys risks sensitive data exposure.
- Unbounded stickiness maps can lead to memory growth.

Mitigations:
- Derive stickiness keys from tokens via a one-way hash (do not store or log raw tokens).
- Do not log requester IPs or token-derived keys.
- Enforce TTL + max entries with eviction.

## Sensitive Data Exposure in Logs

Risk:
- Prompts/messages and API keys are highly sensitive.

Mitigations:
- Log only request metadata and selected chute name (and maybe request id).
- Never log `Authorization` or request body.
