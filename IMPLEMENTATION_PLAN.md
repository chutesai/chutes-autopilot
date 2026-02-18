# Implementation Plan

## Goal
Ship a production-ready Chutes Autopilot router with hardened supply chain, offline/online validation, and CI-enforced quality gates so it can run safely at scale.

## Acceptance Criteria
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and `make smoke` succeed locally and in CI; latest runs are captured under `logs/` (gitignored).
- `research/COVERAGE_MATRIX.md` closes the current gaps on auth/rate-limit/attestation behavior with live findings plus matching offline fixtures; no real secrets are stored.
- Container image builds reproducibly on a pinned toolchain, runs as non-root on a minimal base, and docker-compose healthchecks `/readyz`; README documents runtime UID/GID, ports, and smoke usage.
- `.env.example` and `.gitignore` remain safe (no real keys) while covering new smoke/log artifacts.

## Tasks (ordered by dependency and impact)
- [ ] Offline smoke harness: build a stub upstream + runner (e.g., `scripts/smoke.sh` + `make smoke`) that exercises alias, preference-list, and direct flows (streaming, timeout/failover cases) and writes artifacts to `logs/smoke/` (gitignored).
- [ ] Document smoke usage: wire `make smoke` into README and ensure `.gitignore` covers `logs/smoke/` and any harness outputs under `testdata/smoke/`.
- [ ] Container & compose hardening: refactor `Dockerfile` for pinned Rust toolchain, cached deps, stripped binary, minimal non-root runtime with CA certs; update `docker-compose.yaml` healthcheck to `/readyz` and note runtime user/image expectations in README.
- [ ] CI automation: add `.github/workflows/ci.yml` that runs `make lint`, `cargo test`, and `make smoke` with cargo caching and uploads smoke/log artifacts on failure (optionally gitleaks/trufflehog step guarded for speed).
- [ ] Chutes live verification (provider-specific): run targeted calls against the real backend to capture auth requirements, rate-limit behavior, and `/chutes/{id}/evidence` outputs; record sanitized findings in `research/COVERAGE_MATRIX.md` (and `research/RESEARCH_SUMMARY.md` if needed).
- [ ] Offline parity fixtures from live captures: sanitize and store representative responses under `testdata/chutes_live/`, add integration tests that replay them to validate attestation/rate-limit handling without live network.

## Completed
- [x] Baseline fmt/clippy/test run recorded (2026-02-18) in `logs/baseline.md`.
- [x] Secrets & repo hygiene: gitleaks + trufflehog scans documented in `logs/secrets.md`; `.gitignore` covers logs/secrets and `.env.example` stays secret-free.
- [x] Control plane freshness gating and readiness: `/readyz` fails on missing/stale allowlist or candidate snapshot; metrics expose ages and counts.
- [x] Data-plane correctness and observability: deterministic ranking, alias/direct/list routing, stickiness with rotation, streaming passthrough with pre-commit failover/timeouts/no-retry-after-commit; structured tracing + Prometheus metrics for request/selection/failover; proptests cover model lists, sticky keys, and request-size limits.
