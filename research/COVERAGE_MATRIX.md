# Coverage Matrix

Legend:
- Spec: which spec defines behavior
- Plan: where it appears in `IMPLEMENTATION_PLAN.md`
- Tests: expected test coverage

| Surface / Behavior | Spec | Plan | Tests | Status |
| --- | --- | --- | --- | --- |
| Project foundation (runnable service, tests, lint, README run steps) | `specs/001-project-foundation/spec.md` | Yes | Yes | Implemented |
| Model catalog refresh loop (`MODELS_URL`, refresh interval, last-known-good allowlist) | `specs/003-utilization-and-ranking/spec.md` | Yes | Yes | Implemented |
| Utilization fetch loop (`UTILIZATION_URL`, refresh interval, last-known-good) | `specs/003-utilization-and-ranking/spec.md` | Yes | Yes | Implemented |
| Filtering private chutes (`name == "[private chute]"`) | `specs/003-utilization-and-ranking/spec.md` | Yes | Yes | Implemented |
| Eligibility filtering (filter utilization to chat-capable model allowlist; fallback heuristic when allowlist is empty) | `specs/003-utilization-and-ranking/spec.md` | Yes | Yes | Implemented |
| Ranking algorithm + deterministic tie-breakers | `specs/003-utilization-and-ranking/spec.md` | Yes | Yes | Implemented |
| Snapshot store + hot-path selection | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Alias detection and `model` rewrite | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| User-specified model preference list (comma-separated `model`, ordered failover, allow non-TEE) | `specs/004-user-model-preference-list/spec.md` | Yes | Yes | Implemented |
| Sticky selection + rotation (per client key) | `specs/005-sticky-selection-and-rotation/spec.md` | Yes | Yes | Implemented |
| Proxy upstream request forwarding | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Streaming passthrough (no buffering) | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Failover only before any bytes are sent | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Upstream 429 is non-retryable (proxy back; no failover) | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Timeouts (header + first-body-byte) drive pre-commit failover | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Debug header `x-chutes-autopilot-selected` | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Liveness/readiness endpoints | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Safety: request size limit, no sensitive logging | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented (logging not asserted in tests) |
| Error responses use OpenAI `ErrorResponse` JSON shape | `specs/002-autopilot-mvp/spec.md` | Yes | Yes | Implemented |
| Trusted proxy headers for stickiness (opt-in) | `specs/005-sticky-selection-and-rotation/spec.md` | Yes | Yes | Implemented |
| Live auth + rate-limit semantics for `/v1/chat/completions` (401 JSON on invalid token; 429 HTML when unauthenticated) | `specs/002-autopilot-mvp/spec.md` | Yes | Yes (fixtures `chat_completions_invalid_token_2026-02-18.json`, `chat_completions_no_auth_429_2026-02-18.html`; runtime tests `chat_completions_does_not_failover_on_upstream_401`, `chat_completions_preserves_html_body_on_429`; fixture-contract tests in `tests/live_auth_fixtures.rs`) | Implemented and re-verified in 2026-02-18 test retry (`cargo test --test live_auth_fixtures`) |
| Evidence endpoint behavior (`GET /chutes/{id}/evidence?nonce=` requires 64-hex nonce; non-TEE rejected; older runtime requires `chutes_version >= 0.6.0`) | Research | Yes | Yes (fixtures `evidence_*_2026-02-18.json`; contract + pathological-shape tests in `tests/evidence_fixtures.rs`; live blocker probe fixture `evidence_probe_2026-02-18.json` validated by `tests/evidence_probe_snapshot.rs`) | Implemented and re-verified in 2026-02-18 test retry (`cargo test --test evidence_fixtures --test evidence_probe_snapshot`); success-path still blocked by live runtime versions |

## Gaps / Unknowns

- Evidence success path is still unverified: live probe snapshot (`testdata/chutes_live/evidence_probe_2026-02-18.json`) shows 21/21 sampled TEE chutes returning HTTP 400 with `chutes_version >= 0.6.0` gate text, so no success payload was currently capturable.
- Coverage percentage tooling is not configured in this repo environment (`cargo-llvm-cov`/`cargo-tarpaulin` unavailable); rely on targeted + full regression command evidence until tooling is added.
