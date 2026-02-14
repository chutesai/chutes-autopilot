# 005 - Sticky Selection and Rotation

## Context

Clients want stable model selection when possible (lower jitter) while still preserving reliability. Autopilot should prefer the last-known-good selected model per client and rotate away on retryable failure.

## Requirements

### Client Key

- For each incoming request, compute a `client_key`:
  - If `Authorization: Bearer <token>` is present, `client_key` MUST be derived from the token (store only a hash; never store or log the raw token).
  - Otherwise `client_key` MUST be derived from the requester IP address:
    - Default: the direct TCP peer address (do not trust forwarded headers by default).
    - If `TRUST_PROXY_HEADERS=true` and the direct TCP peer address is in `TRUSTED_PROXY_CIDRS`, use the left-most IP from `X-Forwarded-For` when present.
- Autopilot MUST NOT log auth tokens, auth token hashes, or requester IP addresses.

### Sticky Storage

- Maintain an in-memory mapping `client_key -> sticky_model` with:
  - last-updated time
  - expiry TTL (`STICKY_TTL_SECS`, default `1800`)
  - bounded size (`STICKY_MAX_ENTRIES`, default `10000`)

### Application of Stickiness

- Stickiness applies only when Autopilot is choosing between multiple candidates:
  - AutoPilot alias mode (ranked candidates)
  - explicit model preference list mode (comma-separated list)
- For a given request:
  1. Build the ordered candidate list for the request's routing mode.
  2. If a non-expired `sticky_model` exists for `client_key` AND it exists in the candidate list, move it to the front (preserving relative order of remaining candidates).
  3. Attempt candidates in order using the standard failover boundary (retryable failures only before any response bytes are written).
- When a request succeeds using a model `M`, update `sticky_model` for `client_key` to `M` (refresh TTL).

### Rotation Triggers

- If the current first-choice attempt fails with a retryable failure before any response bytes are written, Autopilot MUST attempt the next candidate.
- Retryable failures for rotation match `specs/002-autopilot-mvp/spec.md` and include:
  - connection errors
  - upstream header timeout (`UPSTREAM_HEADER_TIMEOUT_MS`)
  - upstream `503`
  - upstream 2xx "no body bytes yet" timeout (`UPSTREAM_FIRST_BODY_BYTE_TIMEOUT_MS`)
- Upstream `429` MUST NOT trigger rotation.
- When a later candidate succeeds, Autopilot MUST update `sticky_model` to the successful model.

## Acceptance Criteria

1. **Sticky hit**
   - After a request from a given client selects model `M`, a subsequent request from the same client attempts `M` first if `M` is still eligible.
2. **Sticky miss / ineligible**
   - If the stored `sticky_model` is expired or not present in the current candidate set, Autopilot does not attempt it.
3. **Rotation on retryable failure**
   - If the sticky model attempt fails with a retryable failure before any bytes are written to the client, Autopilot tries the next candidate and updates stickiness on success.
4. **Bounded memory**
   - When `STICKY_MAX_ENTRIES` is exceeded, entries are evicted and new entries can still be inserted.
5. **Forwarded IP trust**
   - When `TRUST_PROXY_HEADERS=false`, Autopilot ignores `X-Forwarded-For` for stickiness and uses the direct TCP peer address for the requester IP.

## Status: COMPLETE
