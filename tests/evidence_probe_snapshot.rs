use std::path::Path;

use serde_json::{json, Value};

fn fixture(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel_path = format!("testdata/chutes_live/{name}");
    let bytes = std::fs::read(root.join(&rel_path)).unwrap_or_else(|e| {
        panic!("failed to read fixture {rel_path}: {e}");
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!("failed to parse fixture {rel_path} as JSON: {e}");
    })
}

fn probe_samples(v: &Value) -> Result<&Vec<Value>, String> {
    let samples = v
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| "fixture must contain array field `samples`".to_string())?;
    if samples.is_empty() {
        return Err("fixture `samples` must not be empty".to_string());
    }
    Ok(samples)
}

#[test]
fn evidence_probe_fixture_documents_current_success_path_blocker() {
    let v = fixture("evidence_probe_2026-02-18.json");
    let nonce_len = v
        .get("nonce_len")
        .and_then(Value::as_u64)
        .expect("probe fixture must include nonce_len");
    assert_eq!(
        nonce_len, 64,
        "probe fixture should use a valid 64-hex nonce"
    );

    let samples = probe_samples(&v).expect("probe fixture contract changed");
    assert!(
        samples.len() >= 20,
        "expected broad TEE probe coverage, got {} samples",
        samples.len()
    );

    for sample in samples {
        let http_status = sample
            .get("http_status")
            .and_then(Value::as_u64)
            .expect("each probe sample must include integer http_status");
        assert_eq!(
            http_status, 400,
            "expected runtime gate (HTTP 400) in probe sample: {sample:?}"
        );
        let detail = sample
            .get("detail")
            .and_then(Value::as_str)
            .expect("each probe sample must include string detail");
        assert!(
            detail.contains("chutes_version >= 0.6.0"),
            "probe detail no longer indicates runtime gate: {detail:?}"
        );
    }
}

#[test]
fn evidence_probe_samples_parser_rejects_pathological_shape() {
    let malformed = json!({
        "samples": {
            "http_status": 400
        }
    });
    let err =
        probe_samples(&malformed).expect_err("parser should reject non-array `samples` shape");
    assert!(
        err.contains("array field `samples`"),
        "unexpected parser error: {err}"
    );
}
