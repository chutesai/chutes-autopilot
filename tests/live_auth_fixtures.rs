use std::path::Path;

use serde_json::{json, Value};

fn fixture_bytes(name: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel_path = format!("testdata/chutes_live/{name}");
    std::fs::read(root.join(&rel_path))
        .unwrap_or_else(|e| panic!("failed to read fixture {rel_path}: {e}"))
}

fn extract_detail_string(v: &Value) -> Result<&str, String> {
    v.get("detail")
        .and_then(Value::as_str)
        .ok_or_else(|| "fixture must contain string field `detail`".to_string())
}

#[test]
fn invalid_token_fixture_is_json_with_expected_detail() {
    let body = fixture_bytes("chat_completions_invalid_token_2026-02-18.json");
    let parsed: Value = serde_json::from_slice(&body)
        .expect("invalid-token fixture should stay valid JSON for 401 passthrough tests");
    let detail =
        extract_detail_string(&parsed).expect("invalid-token fixture detail contract changed");
    assert_eq!(detail, "Invalid token.");
}

#[test]
fn unauthenticated_rate_limit_fixture_is_html_not_json() {
    let body = fixture_bytes("chat_completions_no_auth_429_2026-02-18.html");
    let html = std::str::from_utf8(&body).expect("429 fixture should be valid UTF-8 HTML");
    assert!(
        html.contains("<h1>429 Too Many Requests</h1>"),
        "429 fixture should include canonical rate-limit headline"
    );
    assert!(
        serde_json::from_slice::<Value>(&body).is_err(),
        "429 fixture should remain non-JSON HTML to cover passthrough edge case"
    );
}

#[test]
fn detail_extractor_rejects_pathological_non_string_detail() {
    let malformed = json!({ "detail": 429 });
    let err = extract_detail_string(&malformed)
        .expect_err("extractor should reject non-string `detail` values");
    assert!(
        err.contains("string field `detail`"),
        "unexpected extractor error: {err}"
    );
}
