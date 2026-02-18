# Implementation Plan

## Goal
Prepare chutes-autopilot for production launch with hardened routing, reproducible builds, secret hygiene, and verifiable behavior online and offline.

## Acceptance Criteria
- `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` pass (or failures are captured with repro steps) and results are recorded in `logs/baseline.md`.
- Secrets scan results (working tree + history) are documented in `logs/secrets.md`; `.gitignore` and `.env.example` cover generated logs/caches and contain no sensitive values.
- Control-plane freshness (models allowlist and utilization) is observable and `/readyz` fails when either feed is stale or empty; tests cover fresh/stale/absent cases.
- Property-based and regression tests cover model list parsing/dedup/enforcement, sticky key derivation, request size limits, and streaming failover boundaries (no retries after commit).
- `make smoke` runs offline against a stub backend exercising alias, preference list, and direct model flows; CI hook is present and artifacts land under `logs/smoke/`.
- Live Chutes integration behavior (auth, rate-limit, attestation/evidence) is captured in `research/COVERAGE_MATRIX.md` with stub fixtures so offline runs mirror it.
- Docker image builds reproducibly, runs as non-root on a slim base, and docker-compose healthcheck targets `/readyz`; README documents image expectations and runtime user.

## Tasks (ordered by dependency and impact)
- [ ] Baseline health: run fmt/clippy/test; if anything fails, log details in `logs/baseline.md` with reproduction notes; align `Makefile`/`README.md` so the default path passes locally.
- [ ] Secrets + repo hygiene: run gitleaks/trufflehog on working tree and history; summarize in `logs/secrets.md`; expand `.gitignore`/`.env.example` for new logs/cache artifacts from tests or smoke runs.
- [ ] Control-plane freshness guardrails: track allowlist and utilization refresh timestamps, gate `/readyz` when either is stale or empty, and add tests for fresh/stale/absent combinations.
- [ ] Property-based safety: add `proptest` suites for comma-separated model lists (dedupe, max items), sticky key derivation (auth vs IP, proxy trust), and request size enforcement edge cases.
- [ ] Streaming failover coverage: extend Axum test harness to cover connect timeout/connection reset and confirm no retries after headers or body bytes commit across alias, preference-list, and direct-model requests; assert debug header semantics.
- [ ] Observability hardening: add structured tracing fields (request id, chosen model, snapshot age, failover reason) with sampling that avoids logging sensitive bodies/headers.
- [ ] Offline smoke harness: add a stub upstream plus fixtures and a `make smoke` target (and CI entry) that exercises alias, explicit preference, and direct model flows without external network; store sample outputs under `logs/smoke/`.
- [ ] Chutes integration validation: run against the live backend to document auth, rate-limit, and `/chutes/{id}/evidence` behavior; update `research/COVERAGE_MATRIX.md` and specs with observed deltas.
- [ ] Parity fixtures for Chutes-dependent flows: record or script stub responses for auth errors, rate limits, and attestation so CI and offline runs mirror live findings without hitting Chutes.
- [ ] Container + supply-chain hardening: refactor Dockerfile to cache deps, strip the binary, run as non-root on a minimal base (rustls CA pinned), update docker-compose healthcheck to `/readyz`, and document image size/user expectations in `README.md`.

## Completed
- [x] Implemented ranked candidate refresh with model catalog allowlist and sticky selection, including failover rules for 503/timeouts/429 and OpenAI-compatible error responses.
- [x] Established project scaffolding (Makefile targets, README usage, env defaults) and streaming passthrough with deterministic ranking and tie-breakers.
