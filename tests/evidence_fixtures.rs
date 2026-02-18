use std::path::Path;

use serde_json::Value;

fn fixture(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join("testdata/chutes_live").join(name)).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn evidence_requires_nonce() {
    let v = fixture("evidence_missing_nonce_2026-02-18.json");
    let detail = v.get("detail").unwrap();
    assert!(detail.is_array());
    assert!(detail[0]
        .get("msg")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("required"));
}

#[test]
fn evidence_rejects_short_nonce() {
    let v = fixture("evidence_short_nonce_2026-02-18.json");
    let msg = v.get("detail").and_then(Value::as_str).unwrap();
    assert!(msg.contains("64 hex"));
}

#[test]
fn evidence_rejects_nontee_chute() {
    let v = fixture("evidence_nontee_2026-02-18.json");
    let msg = v.get("detail").and_then(Value::as_str).unwrap();
    assert!(msg.contains("not TEE-enabled"));
}

#[test]
fn evidence_requires_min_runtime_version() {
    let v = fixture("evidence_tee_version_error_2026-02-18.json");
    let msg = v.get("detail").and_then(Value::as_str).unwrap();
    assert!(msg.contains("0.6.0"));
}
