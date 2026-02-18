# Baseline Health Check — 2026-02-18

## Executed Commands (Attempt 2)

1. `cargo test --test evidence_fixtures --test live_auth_fixtures` -> PASS
   - `tests/evidence_fixtures.rs`: 5 passed, 0 failed
   - `tests/live_auth_fixtures.rs`: 3 passed, 0 failed
2. `cargo test` -> PASS
   - `src/lib.rs` unit tests: 55 passed, 0 failed
   - integration tests: 8 passed, 0 failed
   - total: 63 passed, 0 failed
3. `cargo fmt --check` -> PASS
4. `cargo clippy -- -D warnings` -> PASS
5. `make smoke` -> PASS
   - report artifact: `logs/smoke/smoke_1771395024.json`

## Executed Commands (Attempt 3 Retry Repair)

1. `cargo test --test evidence_fixtures --test live_auth_fixtures --test evidence_probe_snapshot` -> PASS
   - `tests/evidence_fixtures.rs`: `5 passed; 0 failed`
   - `tests/live_auth_fixtures.rs`: `3 passed; 0 failed`
   - `tests/evidence_probe_snapshot.rs`: `2 passed; 0 failed`
2. `cargo test` -> PASS
   - `src/lib.rs` unit tests: `55 passed; 0 failed`
   - integration tests: `10 passed; 0 failed`
   - total: `65 passed; 0 failed`
3. `cargo fmt --check` -> PASS (after applying `cargo fmt` once)
4. `cargo clippy -- -D warnings` -> PASS
5. `make smoke` -> PASS
   - report artifact: `logs/smoke/smoke_1771395393.json`
6. Coverage tooling probes:
   - `cargo llvm-cov --version` -> FAIL (`no such command: llvm-cov`)
   - `cargo tarpaulin --version` -> FAIL (`no such command: tarpaulin`)

## Behavior-to-Test Mapping (Attempt 3)

- Live auth/rate-limit fixture contracts:
  - `invalid_token_fixture_is_json_with_expected_detail`
  - `unauthenticated_rate_limit_fixture_is_html_not_json`
  - negative case: `detail_extractor_rejects_pathological_non_string_detail`
- Evidence error-shape contracts:
  - `evidence_requires_nonce`
  - `evidence_rejects_short_nonce`
  - `evidence_rejects_nontee_chute`
  - `evidence_requires_min_runtime_version`
  - negative case: `evidence_missing_nonce_validator_rejects_pathological_shape`
- Evidence success-path blocker (live probe, executable evidence):
  - fixture: `testdata/chutes_live/evidence_probe_2026-02-18.json`
  - assertions: `evidence_probe_fixture_documents_current_success_path_blocker`
  - negative case: `evidence_probe_samples_parser_rejects_pathological_shape`

## High-Risk Surface Verification

- Auth/rate-limit passthrough contracts:
  - invalid token fixture remains JSON with exact `"detail": "Invalid token."`
  - unauthenticated 429 fixture remains HTML (non-JSON) with expected rate-limit header text
- Evidence endpoint contracts:
  - missing nonce validation shape stays stable (`detail[]`, field `nonce`, `"Field required"`)
  - short nonce and min runtime errors keep exact detail strings
  - non-TEE error keeps UUID-bearing message shape
- Pathological/negative cases:
  - missing nonce validator rejects malformed non-array `detail`
  - auth fixture detail extractor rejects non-string `detail`
  - evidence probe parser rejects malformed non-array `samples`

## Coverage Notes

- Coverage tooling is not currently installed/configured in this repo (`cargo-llvm-cov` and `cargo-tarpaulin` unavailable in this environment).
- Remaining known gap: evidence success-path attestation payload is still uncaptured.
- Blocker evidence: live probe across 21 sampled TEE chutes returned HTTP 400 with `Instances requires chutes_version >= 0.6.0 to retrieve evidence.` for each sample (see `testdata/chutes_live/evidence_probe_2026-02-18.json` and `tests/evidence_probe_snapshot.rs`).
