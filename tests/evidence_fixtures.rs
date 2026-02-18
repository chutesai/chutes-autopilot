use std::path::Path;

use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

fn fixture(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel_path = format!("tests/testdata/chutes_live/{name}");
    let bytes = std::fs::read(root.join(&rel_path)).unwrap_or_else(|e| {
        panic!("failed to read fixture {rel_path}: {e}");
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!("failed to parse fixture {rel_path} as JSON: {e}");
    })
}

fn validate_missing_nonce_shape(v: &Value) -> Result<(), String> {
    let detail = v
        .get("detail")
        .ok_or_else(|| "missing top-level `detail`".to_string())?;
    let arr = detail
        .as_array()
        .ok_or_else(|| "`detail` must be an array".to_string())?;
    let first = arr
        .first()
        .ok_or_else(|| "`detail` array must include at least one validation error".to_string())?;

    let msg = first
        .get("msg")
        .and_then(Value::as_str)
        .ok_or_else(|| "`detail[0].msg` must be a string".to_string())?;
    if msg != "Field required" {
        return Err(format!("unexpected detail[0].msg: {msg:?}"));
    }

    let field = first
        .get("loc")
        .and_then(Value::as_array)
        .and_then(|loc| loc.get(1))
        .and_then(Value::as_str)
        .ok_or_else(|| "`detail[0].loc` must include query field name".to_string())?;
    if field != "nonce" {
        return Err(format!("expected missing field `nonce`, got {field:?}"));
    }

    Ok(())
}

#[test]
fn evidence_requires_nonce() {
    let v = fixture("evidence_missing_nonce_2026-02-18.json");
    validate_missing_nonce_shape(&v)
        .unwrap_or_else(|e| panic!("missing nonce fixture contract changed: {e}"));
}

#[test]
fn evidence_rejects_short_nonce() {
    let v = fixture("evidence_short_nonce_2026-02-18.json");
    let msg = v
        .get("detail")
        .and_then(Value::as_str)
        .expect("`detail` should be a string for short nonce errors");
    assert_eq!(
        msg, "Nonce must be exactly 64 hex characters (32 bytes), got 5",
        "short nonce fixture message changed unexpectedly"
    );
}

#[test]
fn evidence_rejects_nontee_chute() {
    let v = fixture("evidence_nontee_2026-02-18.json");
    let msg = v
        .get("detail")
        .and_then(Value::as_str)
        .expect("`detail` should be a string for non-TEE chute errors");
    let prefix = "Chute ";
    let suffix = " is not TEE-enabled";
    assert!(
        msg.starts_with(prefix) && msg.ends_with(suffix),
        "non-TEE message format changed: {msg:?}"
    );
    let chute_id = &msg[prefix.len()..msg.len() - suffix.len()];
    Uuid::parse_str(chute_id)
        .unwrap_or_else(|e| panic!("non-TEE fixture chute id is not a UUID: {e}"));
}

#[test]
fn evidence_requires_min_runtime_version() {
    let v = fixture("evidence_tee_version_error_2026-02-18.json");
    let msg = v
        .get("detail")
        .and_then(Value::as_str)
        .expect("`detail` should be a string for runtime-version errors");
    assert_eq!(
        msg, "Instances requires chutes_version >= 0.6.0 to retrieve evidence.",
        "runtime version guard fixture changed unexpectedly"
    );
}

#[test]
fn evidence_missing_nonce_validator_rejects_pathological_shape() {
    let malformed = json!({
        "detail": {
            "type": "missing",
            "msg": "Field required"
        }
    });
    let err = validate_missing_nonce_shape(&malformed)
        .expect_err("validator should reject non-array detail payloads");
    assert!(
        err.contains("detail") && err.contains("array"),
        "expected array-shape error, got: {err}"
    );
}
