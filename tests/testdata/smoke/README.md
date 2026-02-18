# Smoke Harness Fixtures

The smoke harness (`make smoke`, backed by `src/bin/smoke.rs`) generates its own
stubbed upstream at runtime. It does not currently require committed fixtures in
this directory.

Current smoke scenarios:
- Alias routing with streaming passthrough
- Explicit model-list failover
- First-body-byte timeout failover
- Direct passthrough (no routing header)

The harness waits for Autopilot `/readyz` before executing scenarios.

Smoke artifacts are written to `logs/smoke/`:
- `logs/smoke/latest.json` (latest report)
- `logs/smoke/smoke_<unix_ts>.json` (timestamped history)
