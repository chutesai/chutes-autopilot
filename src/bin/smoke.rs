use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chutes_autopilot::{app, spawn_control_plane_refresh, AppConfig, AppState};
use futures_util::stream::{self, StreamExt};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::Instant;

#[derive(Clone)]
struct StubState {
    attempts: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl StubState {
    fn new() -> Self {
        Self {
            attempts: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    async fn record(&self, model: String) {
        self.attempts.lock().await.push(model);
    }

    async fn take_attempts(&self) -> Vec<String> {
        let mut guard = self.attempts.lock().await;
        let out = guard.clone();
        guard.clear();
        out
    }
}

struct StubServer {
    base_url: String,
    handle: JoinHandle<()>,
    state: StubState,
}

async fn start_stub_server() -> anyhow::Result<StubServer> {
    let state = StubState::new();
    let router = Router::new()
        .route("/v1/models", get(stub_models))
        .route("/chutes/utilization", get(stub_utilization))
        .route("/v1/chat/completions", post(stub_chat_completions))
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");

    let handle = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, router.into_make_service()).await {
            eprintln!("stub server exited: {err:?}");
        }
    });

    Ok(StubServer {
        base_url,
        handle,
        state,
    })
}

#[derive(Clone, Serialize)]
struct ScenarioResult {
    name: String,
    status: u16,
    selected_model: Option<String>,
    body: String,
    attempts: Vec<String>,
    passed: bool,
    notes: Option<String>,
}

#[derive(Serialize)]
struct SmokeReport {
    generated_at: u64,
    stub_base_url: String,
    autopilot_base_url: String,
    results: Vec<ScenarioResult>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_dir = Path::new("logs/smoke");
    fs::create_dir_all(log_dir)?;

    let stub = start_stub_server().await?;
    let autopilot = start_autopilot_server(&stub).await?;

    wait_for_ready(&autopilot.base_url, Duration::from_secs(5)).await?;

    let http = Client::new();

    let alias = run_alias_streaming(&http, &autopilot.base_url, &stub.state).await?;
    let preference = run_preference_failover(&http, &autopilot.base_url, &stub.state).await?;
    let first_byte_timeout =
        run_first_body_timeout_failover(&http, &autopilot.base_url, &stub.state).await?;
    let direct = run_direct_passthrough(&http, &autopilot.base_url, &stub.state).await?;

    let report = SmokeReport {
        generated_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        stub_base_url: stub.base_url.clone(),
        autopilot_base_url: autopilot.base_url.clone(),
        results: vec![alias, preference, first_byte_timeout, direct],
    };

    let all_passed = report.results.iter().all(|r| r.passed);
    if !all_passed {
        eprintln!("some smoke scenarios failed; see logs/smoke/latest.json");
    }

    let artifact = log_dir.join(format!("smoke_{}.json", report.generated_at));
    let pretty = serde_json::to_string_pretty(&report)?;
    fs::write(&artifact, &pretty)?;
    fs::write(log_dir.join("latest.json"), &pretty)?;
    println!("smoke report -> {}", artifact.display());

    autopilot.handle.abort();
    stub.handle.abort();

    if all_passed {
        Ok(())
    } else {
        anyhow::bail!("smoke scenarios failed")
    }
}

async fn start_autopilot_server(stub: &StubServer) -> anyhow::Result<RunningServer> {
    let cfg = AppConfig {
        backend_base_url: stub.base_url.clone(),
        models_url: format!("{}/v1/models", stub.base_url),
        utilization_url: format!("{}/chutes/utilization", stub.base_url),
        models_refresh_ms: Duration::from_millis(200),
        utilization_refresh_ms: Duration::from_millis(200),
        control_plane_timeout: Duration::from_millis(500),
        upstream_header_timeout: Duration::from_millis(120),
        upstream_first_body_byte_timeout: Duration::from_millis(120),
        ..Default::default()
    };

    let state = AppState::new(cfg);
    spawn_control_plane_refresh(state.clone());

    let app = app(state);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");
    let handle = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app.into_make_service()).await {
            eprintln!("autopilot server exited: {err:?}");
        }
    });

    Ok(RunningServer { base_url, handle })
}

struct RunningServer {
    base_url: String,
    handle: JoinHandle<()>,
}

async fn wait_for_ready(base_url: &str, timeout: Duration) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    let http = Client::new();

    loop {
        if Instant::now() > deadline {
            anyhow::bail!("readyz did not succeed within {:?}", timeout);
        }

        match http.get(format!("{base_url}/readyz")).send().await {
            Ok(resp) if resp.status() == StatusCode::OK => return Ok(()),
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
}

async fn run_alias_streaming(
    http: &Client,
    autopilot_url: &str,
    stub_state: &StubState,
) -> anyhow::Result<ScenarioResult> {
    let _ = stub_state.take_attempts().await;
    let resp = http
        .post(format!("{autopilot_url}/v1/chat/completions"))
        .header("content-type", "application/json")
        .body(r#"{"model":"chutesai/AutoPilot","stream":true,"messages":[{"role":"user","content":"hi"}]}"#)
        .send()
        .await?;

    let status = resp.status();
    let selected = resp
        .headers()
        .get("x-chutes-autopilot-selected")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);

    let mut stream = resp.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        body.extend_from_slice(&chunk?);
    }

    let attempts = stub_state.take_attempts().await;
    let body_str = String::from_utf8_lossy(&body).to_string();
    let passed = status == StatusCode::OK
        && selected.as_deref() == Some("alpha-TEE")
        && attempts == vec!["alpha-TEE".to_string()];

    Ok(ScenarioResult {
        name: "alias_streaming".to_string(),
        status: status.as_u16(),
        selected_model: selected,
        body: body_str,
        attempts,
        passed,
        notes: None,
    })
}

async fn run_preference_failover(
    http: &Client,
    autopilot_url: &str,
    stub_state: &StubState,
) -> anyhow::Result<ScenarioResult> {
    let _ = stub_state.take_attempts().await;
    let resp = http
        .post(format!("{autopilot_url}/v1/chat/completions"))
        .header("content-type", "application/json")
        .body(r#"{"model":"slow-TEE,fast-TEE","messages":[{"role":"user","content":"hi"}]}"#)
        .send()
        .await?;

    let status = resp.status();
    let selected = resp
        .headers()
        .get("x-chutes-autopilot-selected")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);
    let bytes = resp.bytes().await?;
    let attempts = stub_state.take_attempts().await;
    let body_str = String::from_utf8_lossy(&bytes).to_string();

    let passed = status == StatusCode::OK
        && selected.as_deref() == Some("fast-TEE")
        && attempts == vec!["slow-TEE".to_string(), "fast-TEE".to_string()];

    Ok(ScenarioResult {
        name: "preference_failover".to_string(),
        status: status.as_u16(),
        selected_model: selected,
        body: body_str,
        attempts,
        passed,
        notes: None,
    })
}

async fn run_first_body_timeout_failover(
    http: &Client,
    autopilot_url: &str,
    stub_state: &StubState,
) -> anyhow::Result<ScenarioResult> {
    let _ = stub_state.take_attempts().await;
    let resp = http
        .post(format!("{autopilot_url}/v1/chat/completions"))
        .header("content-type", "application/json")
        .body(r#"{"model":"stall-TEE,fast-TEE","messages":[{"role":"user","content":"hi"}]}"#)
        .send()
        .await?;

    let status = resp.status();
    let selected = resp
        .headers()
        .get("x-chutes-autopilot-selected")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);
    let bytes = resp.bytes().await?;
    let attempts = stub_state.take_attempts().await;
    let body_str = String::from_utf8_lossy(&bytes).to_string();

    let passed = status == StatusCode::OK
        && selected.as_deref() == Some("fast-TEE")
        && attempts == vec!["stall-TEE".to_string(), "fast-TEE".to_string()]
        && body_str == "fast-path";

    Ok(ScenarioResult {
        name: "first_body_timeout_failover".to_string(),
        status: status.as_u16(),
        selected_model: selected,
        body: body_str,
        attempts,
        passed,
        notes: None,
    })
}

async fn run_direct_passthrough(
    http: &Client,
    autopilot_url: &str,
    stub_state: &StubState,
) -> anyhow::Result<ScenarioResult> {
    let _ = stub_state.take_attempts().await;
    let resp = http
        .post(format!("{autopilot_url}/v1/chat/completions"))
        .header("content-type", "application/json")
        .body(r#"{"model":"direct-nontee","messages":[{"role":"user","content":"hi"}]}"#)
        .send()
        .await?;

    let selected = resp
        .headers()
        .get("x-chutes-autopilot-selected")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);
    let status = resp.status();
    let bytes = resp.bytes().await?;
    let attempts = stub_state.take_attempts().await;
    let body_str = String::from_utf8_lossy(&bytes).to_string();

    let passed = status == StatusCode::CREATED
        && selected.is_none()
        && attempts == vec!["direct-nontee".to_string()]
        && body_str == "direct-ok";

    Ok(ScenarioResult {
        name: "direct_passthrough".to_string(),
        status: status.as_u16(),
        selected_model: selected,
        body: body_str,
        attempts,
        passed,
        notes: None,
    })
}

async fn stub_models() -> impl axum::response::IntoResponse {
    Json(serde_json::json!({
        "data": [
            {"id": "alpha-TEE"},
            {"id": "beta-TEE"},
            {"id": "slow-TEE"},
            {"id": "fast-TEE"},
            {"id": "stall-TEE"},
            {"id": "direct-nontee"},
        ]
    }))
}

async fn stub_utilization() -> impl axum::response::IntoResponse {
    Json(vec![
        serde_json::json!({
            "name": "alpha-TEE",
            "active_instance_count": 8,
            "utilization_current": 0.1,
            "rate_limit_ratio_5m": 0.0,
            "scalable": true,
            "scale_allowance": 4.0
        }),
        serde_json::json!({
            "name": "beta-TEE",
            "active_instance_count": 4,
            "utilization_current": 0.2,
            "rate_limit_ratio_5m": 0.0,
            "scalable": false
        }),
        serde_json::json!({
            "name": "slow-TEE",
            "active_instance_count": 6,
            "utilization_current": 0.15,
            "rate_limit_ratio_5m": 0.0,
            "scalable": false
        }),
        serde_json::json!({
            "name": "fast-TEE",
            "active_instance_count": 2,
            "utilization_current": 0.05,
            "rate_limit_ratio_5m": 0.0,
            "scalable": false
        }),
        serde_json::json!({
            "name": "stall-TEE",
            "active_instance_count": 3,
            "utilization_current": 0.05,
            "rate_limit_ratio_5m": 0.0,
            "scalable": false
        }),
    ])
}

async fn stub_chat_completions(
    axum::extract::State(state): axum::extract::State<StubState>,
    Json(body): Json<Value>,
) -> Response {
    let model = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    state.record(model.clone()).await;

    match model.as_str() {
        "alpha-TEE" => {
            let first = stream::once(async move {
                Ok::<Bytes, std::io::Error>(Bytes::from_static(b"alpha-first"))
            });
            let second = stream::once(async move {
                tokio::time::sleep(Duration::from_millis(40)).await;
                Ok::<Bytes, std::io::Error>(Bytes::from_static(b"alpha-second"))
            });
            let combined = first.chain(second);
            let mut resp = Response::new(Body::from_stream(combined));
            *resp.status_mut() = StatusCode::OK;
            resp
        }
        "slow-TEE" => {
            tokio::time::sleep(Duration::from_millis(300)).await;
            (StatusCode::OK, "slow-path").into_response()
        }
        "fast-TEE" => (StatusCode::OK, "fast-path").into_response(),
        "stall-TEE" => {
            let delayed = stream::once(async move {
                tokio::time::sleep(Duration::from_millis(300)).await;
                Ok::<Bytes, std::io::Error>(Bytes::from_static(b"late"))
            });
            let mut resp = Response::new(Body::from_stream(delayed));
            *resp.status_mut() = StatusCode::OK;
            resp
        }
        "direct-nontee" => (StatusCode::CREATED, "direct-ok").into_response(),
        _ => (StatusCode::OK, "ok").into_response(),
    }
}
