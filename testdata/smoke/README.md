# Smoke Harness Fixtures

The smoke harness (`make smoke`, backed by `src/bin/smoke.rs`) generates its own
stubbed upstream at runtime. It does not currently require committed fixtures in
this directory.

Smoke artifacts are written to `logs/smoke/`:
- `logs/smoke/latest.json` (latest report)
- `logs/smoke/smoke_<unix_ts>.json` (timestamped history)
