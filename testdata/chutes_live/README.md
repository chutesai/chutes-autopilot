# Chutes Live Fixtures (Captured 2026-02-18)

Purpose: provide offline parity for parsing/routing decisions without hitting live Chutes endpoints.

Sources:
- `models_2026-02-18.json`: `GET https://llm.chutes.ai/v1/models` (no auth required).
- `utilization_2026-02-18.json`: `GET https://api.chutes.ai/chutes/utilization` (no auth required).
- `evidence_*`: `GET https://api.chutes.ai/chutes/{id}/evidence?nonce=...` with different chutes/nonces to document attestation behavior.
- `evidence_probe_2026-02-18.json`: live probe snapshot across 21 TEE chutes using a valid 64-hex nonce; records runtime versions and evidence endpoint responses.
- `chat_completions_invalid_token_2026-02-18.json`: `POST /v1/chat/completions` with `Authorization: Bearer invalid-token` (401 JSON body).
- `chat_completions_no_auth_429_2026-02-18.html`: `POST /v1/chat/completions` without auth (nginx 429 HTML).

Notes:
- Nonces are synthetic and contain no secrets.
- Evidence responses show: missing nonce -> 422; short nonce -> 400; non-TEE chute -> 400 "not TEE-enabled"; TEE chute but older runtime -> 400 requiring `chutes_version >= 0.6.0`.
- Probe snapshot confirms no sampled TEE chute had `chutes_version >= 0.6.0` at capture time, so success-path evidence payloads were unavailable.
- These files are safe to commit; keep future captures date-stamped.
