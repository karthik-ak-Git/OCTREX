use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use octrex_core::{
    agent::AgentExecutionRequest, AppConfig, ProviderGateway, WorkspaceManager,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

struct AppState {
    config: Mutex<AppConfig>,
    gateway: ProviderGateway,
    agent_engine: octrex_core::AgentEngine,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = AppConfig::load();
    let gateway = ProviderGateway::new();
    let agent_engine = octrex_core::AgentEngine::new();

    let state = Arc::new(AppState {
        config: Mutex::new(config),
        gateway,
        agent_engine,
    });

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/providers", get(get_providers_handler))
        .route("/api/providers/{id}/connect", post(connect_provider_handler))
        .route("/api/workspace/inspect", post(inspect_workspace_handler))
        .route("/api/agent/execute", post(execute_agent_handler))
        .nest_service("/ui", ServeDir::new("ui"))
        .route("/", get(serve_ui_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("Invalid socket address");

    println!("\n================================================================");
    println!("       OCTREX CODE V4 — NATIVE RUST BACKEND ENGINE            ");
    println!("================================================================");
    println!("  ➜ Local Server:  http://{}", addr);
    println!("  ➜ UI Endpoint:   http://{}/", addr);
    println!("  ➜ Provider Check: Real API requests (Zero dummy/mock data)");
    println!("================================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_ui_handler() -> impl IntoResponse {
    match std::fs::read_to_string("ui/index.html") {
        Ok(content) => Html(content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "UI file index.html not found").into_response(),
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "HEALTHY",
        "engine": "octrex-core-rust",
        "version": "4.0.0"
    }))
}

async fn get_providers_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let cfg = {
        let guard = state.config.lock().unwrap();
        guard.clone()
    };

    let checks = state.gateway.check_all(&cfg).await;
    Json(serde_json::json!({ "providers": checks }))
}

#[derive(Deserialize)]
struct ConnectPayload {
    api_key: String,
}

#[axum::debug_handler]
async fn connect_provider_handler(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ConnectPayload>,
) -> impl IntoResponse {
    let provider_cfg = {
        let mut guard = match state.config.lock() {
            Ok(g) => g,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

        if let Err(e) = guard.set_api_key(&provider_id, payload.api_key.clone()) {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }

        match guard.providers.get(&provider_id).cloned() {
            Some(cfg) => cfg,
            None => return (StatusCode::NOT_FOUND, "Provider not found".to_string()).into_response(),
        }
    };

    let check = state
        .gateway
        .check_health(&provider_id, &provider_cfg)
        .await;

    Json(serde_json::json!({
        "success": true,
        "health": check
    }))
    .into_response()
}

#[derive(Deserialize)]
struct InspectWorkspacePayload {
    path: String,
}

async fn inspect_workspace_handler(
    Json(payload): Json<InspectWorkspacePayload>,
) -> impl IntoResponse {
    let info = WorkspaceManager::inspect(PathBuf::from(payload.path));
    Json(info)
}

#[derive(Deserialize)]
struct ExecuteAgentPayload {
    prompt: String,
    provider_id: Option<String>,
    workspace_path: Option<String>,
}

async fn execute_agent_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExecuteAgentPayload>,
) -> impl IntoResponse {
    let cfg = {
        let guard = match state.config.lock() {
            Ok(g) => g,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
        guard.clone()
    };

    let req = AgentExecutionRequest {
        prompt: payload.prompt,
        provider_id: payload.provider_id,
        workspace_path: payload.workspace_path,
    };

    match state.agent_engine.execute(&cfg, req).await {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}
