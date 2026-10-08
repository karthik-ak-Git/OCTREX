use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Router,
};
use futures_util::stream::Stream;
use octrex_core::{
    agent::AgentExecutionRequest,
    app::ApplicationState,
    events::{
        types::{
            FileChangedPayload, TaskCompletedPayload, TaskCreatedPayload, TaskFailedPayload,
            TaskStartedPayload,
        },
        EventEnvelope, EventType,
    },
    ids::{RequestId, WorkspaceId},
    ipc::IpcCommandHandler,
    models::ModelRequirement,
    providers::ExecutionMode,
    WorkspaceManager,
};
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

pub struct ServerState {
    pub app: ApplicationState,
    pub start_time: Instant,
}

#[tokio::main]
async fn main() {
    let app_state = ApplicationState::initialize();
    let start_time = Instant::now();

    let state = Arc::new(ServerState {
        app: app_state,
        start_time,
    });

    let app = Router::new()
        .route("/api/info", get(info_handler))
        .route("/api/status", get(status_handler))
        .route("/api/health", get(health_handler))
        .route("/api/db/status", get(db_status_handler))
        .route("/api/config/summary", get(config_summary_handler))
        .route("/api/events", get(events_sse_handler))
        .route("/api/providers", get(get_providers_handler))
        .route("/api/providers/:id", get(get_provider_by_id_handler))
        .route("/api/providers/:id/connect", post(connect_provider_handler))
        .route(
            "/api/providers/:id/models",
            get(get_provider_models_handler),
        )
        .route("/api/models", get(get_models_handler))
        .route("/api/models/*id", get(get_model_by_id_handler))
        .route("/api/hardware", get(get_hardware_handler))
        .route(
            "/api/hardware/refresh",
            get(refresh_hardware_handler).post(refresh_hardware_handler),
        )
        .route(
            "/api/hardware/compatibility",
            get(get_hardware_compatibility_handler),
        )
        .route("/api/workspace/inspect", post(inspect_workspace_handler))
        .route("/api/workspace/tree", post(tree_workspace_handler))
        .route("/api/workspace/read_file", post(read_file_handler))
        .route("/api/workspace/write_file", post(write_file_handler))
        .route("/api/agent/execute", post(execute_agent_handler))
        .route("/api/network/status", get(get_network_status_handler))
        .route("/api/network/policy", get(get_network_policy_handler))
        .route("/api/network/rules", get(get_network_rules_handler))
        .route("/api/network/evaluate", post(evaluate_network_handler))
        .route("/api/network/test", post(test_network_handler))
        .route("/api/network/decisions", get(get_network_decisions_handler))
        .route(
            "/api/network/decisions/:id",
            get(get_network_decision_by_id_handler),
        )
        .route("/api/network/consent", post(network_consent_handler))
        .route("/api/network/allowlist", post(add_allowlist_handler))
        .route(
            "/api/network/allowlist/:id",
            axum::routing::delete(delete_allowlist_handler),
        )
        .route("/api/network/refresh", post(refresh_network_handler))
        .route("/api/network/mode", post(set_network_mode_handler))
        .route("/api/privacy/status", get(get_privacy_status_handler))
        .route(
            "/api/privacy/settings",
            get(get_privacy_settings_handler).post(update_privacy_settings_handler),
        )
        .route("/api/privacy/policies", get(get_privacy_policies_handler))
        .route("/api/privacy/evaluate", post(evaluate_privacy_handler))
        .route("/api/privacy/consent", post(submit_privacy_consent_handler))
        .nest_service("/assets", ServeDir::new("ui/assets"))
        .fallback_service(
            ServeDir::new("apps/web/out").fallback(ServeFile::new("apps/web/out/index.html")),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("Invalid socket address");

    println!("\n================================================================");
    println!("       OCTREX CODE V4 — RUST APPLICATION CORE RUNTIME          ");
    println!("================================================================");
    println!("  ➜ Local Server:  http://{}", addr);
    println!("  ➜ Next.js UI:    http://{}/", addr);
    println!("  ➜ API Info:      http://{}/api/info", addr);
    println!("  ➜ SSE Events:    http://{}/api/events", addr);
    println!("================================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn info_handler() -> Json<serde_json::Value> {
    let info = IpcCommandHandler::get_application_info();
    Json(serde_json::to_value(info).unwrap())
}

async fn status_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let status = IpcCommandHandler::get_runtime_status(&state.app, state.start_time);
    Json(serde_json::to_value(status).unwrap())
}

async fn health_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let health = IpcCommandHandler::get_backend_health(&state.app);
    Json(serde_json::to_value(health).unwrap())
}

async fn db_status_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let status = state.app.db.get_status();
    Json(serde_json::to_value(status).unwrap())
}

async fn config_summary_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let summary = IpcCommandHandler::get_configuration_summary(&state.app);
    Json(serde_json::to_value(summary).unwrap())
}

async fn events_sse_handler(
    State(state): State<Arc<ServerState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.app.event_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| match item {
        Ok(envelope) => {
            let json = serde_json::to_string(&envelope).unwrap_or_default();
            Some(Ok(Event::default().data(json)))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_providers_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let cfg = {
        let guard = state.app.config.read().unwrap();
        guard.clone()
    };

    let checks = state.app.provider_gateway.check_all(&cfg).await;
    let registered_providers = state.app.provider_registry.list_providers();

    Json(serde_json::json!({
        "providers": checks,
        "descriptors": registered_providers
    }))
}

async fn get_provider_by_id_handler(
    State(state): State<Arc<ServerState>>,
    Path(provider_id): Path<String>,
) -> impl IntoResponse {
    match state.app.provider_registry.get_provider(&provider_id) {
        Some(provider) => Json(serde_json::json!({
            "success": true,
            "provider": provider.provider_info(),
            "capabilities": provider.capabilities()
        }))
        .into_response(),
        None => Json(serde_json::json!({
            "success": false,
            "error": format!("Provider '{}' not found", provider_id)
        }))
        .into_response(),
    }
}

async fn get_provider_models_handler(
    State(state): State<Arc<ServerState>>,
    Path(provider_id): Path<String>,
) -> impl IntoResponse {
    let models = state.app.model_registry.filter_by_provider(&provider_id);
    Json(serde_json::json!({
        "success": true,
        "provider_id": provider_id,
        "models": models
    }))
}

#[derive(Deserialize)]
struct ModelsQuery {
    execution_mode: Option<String>,
    capability: Option<String>,
}

async fn get_models_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ModelsQuery>,
) -> Json<serde_json::Value> {
    let mut req = ModelRequirement::new();

    if let Some(mode_str) = &query.execution_mode {
        match mode_str.to_lowercase().as_str() {
            "local" => req = req.with_execution_mode(ExecutionMode::Local),
            "on_premise" => req = req.with_execution_mode(ExecutionMode::OnPremise),
            "cloud" => req = req.with_execution_mode(ExecutionMode::Cloud),
            _ => {}
        }
    }

    if let Some(cap_str) = &query.capability {
        match cap_str.to_lowercase().as_str() {
            "vision" => req = req.require_vision(),
            "tool_calling" => req = req.require_tool_calling(),
            "structured_output" => req = req.require_structured_output(),
            _ => {}
        }
    }

    let models = if query.execution_mode.is_some() || query.capability.is_some() {
        state.app.model_registry.filter_by_requirement(&req)
    } else {
        state.app.model_registry.list_models()
    };

    Json(serde_json::json!({
        "success": true,
        "total": models.len(),
        "models": models
    }))
}

async fn get_model_by_id_handler(
    State(state): State<Arc<ServerState>>,
    Path(model_id): Path<String>,
) -> impl IntoResponse {
    match state.app.model_registry.get_model(&model_id) {
        Some(model) => Json(serde_json::json!({
            "success": true,
            "model": model
        }))
        .into_response(),
        None => Json(serde_json::json!({
            "success": false,
            "error": format!("Model '{}' not found", model_id)
        }))
        .into_response(),
    }
}

#[derive(Deserialize)]
struct ConnectPayload {
    api_key: String,
}

async fn connect_provider_handler(
    State(state): State<Arc<ServerState>>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ConnectPayload>,
) -> impl IntoResponse {
    let provider_cfg = {
        let mut guard = match state.app.config.write() {
            Ok(g) => g,
            Err(e) => {
                return Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                    .into_response()
            }
        };

        if let Err(e) = guard.set_api_key(&provider_id, payload.api_key.clone()) {
            return Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                .into_response();
        }

        match guard.providers.get(&provider_id).cloned() {
            Some(cfg) => cfg,
            None => {
                return Json(serde_json::json!({
                    "success": false,
                    "error": format!("Provider '{}' not found", provider_id)
                }))
                .into_response()
            }
        }
    };

    // Re-register updated provider adapter
    state
        .app
        .provider_gateway
        .register_from_config(&provider_id, &provider_cfg)
        .await;

    let check = state
        .app
        .provider_gateway
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
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<InspectWorkspacePayload>,
) -> impl IntoResponse {
    let path = PathBuf::from(&payload.path);
    let info = WorkspaceManager::inspect(&path);
    let ws = state
        .app
        .workspace_registry
        .register_workspace(&info.name, path);
    Json(serde_json::json!({
        "workspace_id": ws.id.as_str(),
        "info": info
    }))
}

#[derive(Deserialize)]
struct WorkspaceTreePayload {
    path: String,
}

async fn tree_workspace_handler(Json(payload): Json<WorkspaceTreePayload>) -> impl IntoResponse {
    let tree = WorkspaceManager::list_tree(PathBuf::from(payload.path));
    Json(serde_json::json!({
        "success": true,
        "entries": tree
    }))
}

#[derive(Deserialize)]
struct ReadFilePayload {
    workspace_path: String,
    rel_path: String,
}

async fn read_file_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ReadFilePayload>,
) -> impl IntoResponse {
    match WorkspaceManager::read_file(PathBuf::from(&payload.workspace_path), &payload.rel_path) {
        Ok(content) => {
            let event = EventEnvelope::new(
                EventType::FileRead,
                serde_json::json!({
                    "workspace_path": payload.workspace_path,
                    "rel_path": payload.rel_path,
                    "bytes": content.len()
                }),
            );
            let _ = state.app.event_bus.publish(event);
            Json(serde_json::json!({ "success": true, "content": content })).into_response()
        }
        Err(err) => {
            Json(serde_json::json!({ "success": false, "error": err.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct WriteFilePayload {
    workspace_path: String,
    rel_path: String,
    content: String,
}

async fn write_file_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<WriteFilePayload>,
) -> impl IntoResponse {
    match WorkspaceManager::write_file(
        PathBuf::from(&payload.workspace_path),
        &payload.rel_path,
        &payload.content,
    ) {
        Ok(bytes) => {
            let file_event = EventEnvelope::new(
                EventType::FileChanged,
                FileChangedPayload {
                    path: format!("{}/{}", payload.workspace_path, payload.rel_path),
                    rel_path: payload.rel_path.clone(),
                    action: "modified".to_string(),
                },
            );
            let _ = state.app.event_bus.publish(file_event);

            Json(serde_json::json!({ "success": true, "bytes_written": bytes })).into_response()
        }
        Err(err) => {
            Json(serde_json::json!({ "success": false, "error": err.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct ExecuteAgentPayload {
    prompt: String,
    provider_id: Option<String>,
    workspace_path: Option<String>,
}

async fn execute_agent_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ExecuteAgentPayload>,
) -> impl IntoResponse {
    let req_id = RequestId::new();
    let ws_id = payload
        .workspace_path
        .as_ref()
        .map(|p| WorkspaceId::from_string(p));

    // Create & track task
    let task = state.app.task_registry.create_task(
        &payload.prompt,
        Some(req_id.clone()),
        None,
        ws_id.clone(),
    );

    // Publish TASK_CREATED event
    let _ = state.app.event_bus.publish(
        EventEnvelope::new(
            EventType::TaskCreated,
            TaskCreatedPayload {
                task_id: task.id.to_string(),
                title: task.title.clone(),
                workspace_path: payload.workspace_path.clone(),
            },
        )
        .with_request_id(req_id.clone())
        .with_task_id(task.id.clone()),
    );

    let provider_id = payload
        .provider_id
        .clone()
        .unwrap_or_else(|| state.app.config.read().unwrap().active_provider.clone());

    // Update & publish TASK_STARTED
    state
        .app
        .task_registry
        .update_status(&task.id, octrex_core::task::TaskStatus::Running, None);
    let _ = state.app.event_bus.publish(
        EventEnvelope::new(
            EventType::TaskStarted,
            TaskStartedPayload {
                task_id: task.id.to_string(),
                provider_id: provider_id.clone(),
            },
        )
        .with_request_id(req_id.clone())
        .with_task_id(task.id.clone()),
    );

    let cfg = {
        let guard = match state.app.config.read() {
            Ok(g) => g,
            Err(e) => {
                return Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                    .into_response()
            }
        };
        guard.clone()
    };

    let req = AgentExecutionRequest {
        prompt: payload.prompt,
        provider_id: payload.provider_id,
        workspace_path: payload.workspace_path,
    };

    match state.app.agent_engine.execute(&cfg, req).await {
        Ok(resp) => {
            state.app.task_registry.update_status(
                &task.id,
                octrex_core::task::TaskStatus::Completed,
                None,
            );

            let _ = state.app.event_bus.publish(
                EventEnvelope::new(
                    EventType::TaskCompleted,
                    TaskCompletedPayload {
                        task_id: task.id.to_string(),
                        output_summary: if resp.output.len() > 100 {
                            format!("{}...", &resp.output[..100])
                        } else {
                            resp.output.clone()
                        },
                        execution_time_ms: resp.execution_time_ms,
                    },
                )
                .with_request_id(req_id)
                .with_task_id(task.id.clone()),
            );

            Json(serde_json::json!({
                "success": true,
                "task_id": task.id.to_string(),
                "provider_used": resp.provider_used,
                "output": resp.output,
                "execution_time_ms": resp.execution_time_ms
            }))
            .into_response()
        }
        Err(err) => {
            let err_resp = octrex_core::error::OctrexError::Provider {
                provider_id,
                message: err.to_string(),
            }
            .to_response(Some(req_id.clone()));

            state.app.task_registry.update_status(
                &task.id,
                octrex_core::task::TaskStatus::Failed,
                Some(err_resp.clone()),
            );

            let _ = state.app.event_bus.publish(
                EventEnvelope::new(
                    EventType::TaskFailed,
                    TaskFailedPayload {
                        task_id: task.id.to_string(),
                        error: err.to_string(),
                    },
                )
                .with_request_id(req_id)
                .with_task_id(task.id),
            );

            Json(serde_json::json!({
                "success": false,
                "error": err_resp
            }))
            .into_response()
        }
    }
}

async fn get_hardware_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    match state.app.hardware_service.profile().await {
        Ok(profile) => {
            let snapshot = state.app.hardware_service.snapshot().await.ok();
            Json(serde_json::json!({
                "success": true,
                "profile": profile,
                "snapshot": snapshot
            }))
            .into_response()
        }
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        }))
        .into_response(),
    }
}

async fn refresh_hardware_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    match state.app.hardware_service.refresh().await {
        Ok(profile) => Json(serde_json::json!({
            "success": true,
            "profile": profile
        }))
        .into_response(),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        }))
        .into_response(),
    }
}

#[derive(Deserialize)]
struct HardwareCompatibilityQuery {
    model_id: Option<String>,
}

async fn get_hardware_compatibility_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<HardwareCompatibilityQuery>,
) -> impl IntoResponse {
    let profile = match state.app.hardware_service.profile().await {
        Ok(p) => p,
        Err(e) => {
            return Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
            .into_response()
        }
    };

    if let Some(model_id) = &query.model_id {
        match state.app.model_registry.get_model(model_id) {
            Some(model) => {
                let eval = state
                    .app
                    .hardware_service
                    .evaluate_compatibility(&profile, &model);
                Json(serde_json::json!({
                    "success": true,
                    "model_id": model_id,
                    "compatibility": eval
                }))
                .into_response()
            }
            None => Json(serde_json::json!({
                "success": false,
                "error": format!("Model '{}' not found in registry", model_id)
            }))
            .into_response(),
        }
    } else {
        let models = state.app.model_registry.list_models();
        let evals: Vec<_> = models
            .iter()
            .map(|m| {
                state
                    .app
                    .hardware_service
                    .evaluate_compatibility(&profile, m)
            })
            .collect();

        Json(serde_json::json!({
            "success": true,
            "total": evals.len(),
            "compatibilities": evals
        }))
        .into_response()
    }
}

async fn get_network_status_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let status = state.app.network_security.get_status();
    Json(serde_json::to_value(status).unwrap())
}

async fn get_network_policy_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let mode = state.app.network_security.get_mode();
    let rules = state.app.network_security.get_rules();
    let allowlist = state.app.network_security.get_allowlist();

    Json(serde_json::json!({
        "success": true,
        "mode": mode,
        "policy_hierarchy": [
            "1. System Policy (Priority 1)",
            "2. Company Policy (Priority 2)",
            "3. Security Policy (Priority 3)",
            "4. Privacy Policy (Priority 4)",
            "5. Permission Policy (Priority 5)",
            "6. User Policy (Priority 6)"
        ],
        "fail_closed": true,
        "active_rules": rules,
        "allowlist": allowlist
    }))
}

async fn get_network_rules_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let rules = state.app.network_security.get_rules();
    Json(serde_json::json!({
        "success": true,
        "total": rules.len(),
        "rules": rules
    }))
}

#[derive(Deserialize)]
struct EvaluateNetworkPayload {
    destination: String,
    capability: Option<String>,
    source: Option<String>,
    method: Option<String>,
}

async fn evaluate_network_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<EvaluateNetworkPayload>,
) -> impl IntoResponse {
    let ep = match octrex_core::NetworkEndpoint::parse(&payload.destination) {
        Ok(ep) => ep,
        Err(err) => {
            return Json(serde_json::json!({
                "success": false,
                "error": err.to_string(),
                "disposition": "block",
                "reason": "Invalid destination URL"
            }))
            .into_response()
        }
    };

    let cap = match payload.capability.as_deref() {
        Some("cloud_model_inference") => octrex_core::NetworkCapability::CloudModelInference,
        Some("web_fetch") => octrex_core::NetworkCapability::WebFetch,
        Some("web_search") => octrex_core::NetworkCapability::WebSearch,
        Some("remote_mcp") => octrex_core::NetworkCapability::RemoteMcp,
        Some("provider_api") => octrex_core::NetworkCapability::ProviderApi,
        Some("local_network") => octrex_core::NetworkCapability::LocalNetwork,
        Some("loopback") => octrex_core::NetworkCapability::Loopback,
        _ => octrex_core::NetworkCapability::ExternalHttps,
    };

    let source = payload
        .source
        .unwrap_or_else(|| "api_evaluation".to_string());
    let req = octrex_core::NetworkRequest::new(source, cap, ep)
        .with_method(payload.method.unwrap_or_else(|| "GET".to_string()));

    let decision = state.app.network_security.evaluate_request(&req);
    Json(serde_json::json!({
        "success": true,
        "decision": decision
    }))
    .into_response()
}

async fn test_network_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<EvaluateNetworkPayload>,
) -> impl IntoResponse {
    evaluate_network_handler(State(state), Json(payload)).await
}

async fn get_network_decisions_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let decisions = state.app.network_security.get_decisions();
    Json(serde_json::json!({
        "success": true,
        "total": decisions.len(),
        "decisions": decisions
    }))
}

async fn get_network_decision_by_id_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.network_security.get_decision_by_id(&id) {
        Some(decision) => Json(serde_json::json!({
            "success": true,
            "decision": decision
        }))
        .into_response(),
        None => Json(serde_json::json!({
            "success": false,
            "error": format!("Decision with ID '{}' not found", id)
        }))
        .into_response(),
    }
}

#[derive(Deserialize)]
struct ConsentPayload {
    request_id: String,
    destination: String,
    granted: bool,
}

async fn network_consent_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ConsentPayload>,
) -> impl IntoResponse {
    let record = state.app.network_security.record_consent(
        &payload.request_id,
        &payload.destination,
        payload.granted,
    );
    Json(serde_json::json!({
        "success": true,
        "consent": record
    }))
}

#[derive(Deserialize)]
struct AllowlistPayload {
    domain_pattern: String,
    description: Option<String>,
}

async fn add_allowlist_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<AllowlistPayload>,
) -> impl IntoResponse {
    let desc = payload
        .description
        .unwrap_or_else(|| "User added allowlist entry".to_string());
    state
        .app
        .network_security
        .add_allowlist_entry(&payload.domain_pattern, &desc);
    Json(serde_json::json!({
        "success": true,
        "message": format!("Added '{}' to allowlist", payload.domain_pattern)
    }))
}

async fn delete_allowlist_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let removed = state.app.network_security.remove_allowlist_entry(&id);
    Json(serde_json::json!({
        "success": removed,
        "message": if removed { format!("Removed allowlist entry '{}'", id) } else { format!("Allowlist entry '{}' not found", id) }
    }))
}

async fn refresh_network_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let status = state.app.network_security.get_status();
    Json(serde_json::json!({
        "success": true,
        "status": status
    }))
}

#[derive(Deserialize)]
struct SetNetworkModePayload {
    mode: String,
}

async fn set_network_mode_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<SetNetworkModePayload>,
) -> impl IntoResponse {
    let mode = match payload.mode.to_lowercase().as_str() {
        "local_only" | "localonly" => octrex_core::NetworkMode::LocalOnly,
        "restricted" => octrex_core::NetworkMode::Restricted,
        "online_allowed" | "onlineallowed" => octrex_core::NetworkMode::OnlineAllowed,
        "disabled" => octrex_core::NetworkMode::Disabled,
        _ => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Invalid mode '{}'. Must be one of: local_only, restricted, online_allowed, disabled", payload.mode)
            })).into_response();
        }
    };

    state.app.network_security.set_mode(mode);
    Json(serde_json::json!({
        "success": true,
        "mode": mode,
        "status": state.app.network_security.get_status()
    }))
    .into_response()
}

async fn get_privacy_status_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let cfg = state.app.config.read().unwrap();
    let status = state.app.privacy_gate.get_effective_status(
        cfg.privacy
            .mode
            .parse()
            .unwrap_or(octrex_core::privacy::PrivacyMode::LocalOnly),
        cfg.privacy.confidential_mode,
        cfg.active_workspace
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        octrex_core::privacy::PrivacyClassification::Public,
    );
    Json(serde_json::to_value(status).unwrap())
}

async fn get_privacy_settings_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let cfg = state.app.config.read().unwrap();
    Json(serde_json::json!({
        "privacy_mode": cfg.privacy.mode,
        "confidential_mode": cfg.privacy.confidential_mode,
        "consent_behavior": cfg.privacy.consent_behavior,
        "enforcement_level": cfg.privacy.enforcement_level,
    }))
}

#[derive(Deserialize)]
struct UpdatePrivacySettingsPayload {
    privacy_mode: Option<String>,
    confidential_mode: Option<bool>,
}

async fn update_privacy_settings_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<UpdatePrivacySettingsPayload>,
) -> impl IntoResponse {
    let mut cfg = match state.app.config.write() {
        Ok(g) => g,
        Err(e) => {
            return Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                .into_response()
        }
    };
    if let Some(mode) = payload.privacy_mode {
        cfg.privacy.mode = mode;
    }
    if let Some(confidential) = payload.confidential_mode {
        cfg.privacy.confidential_mode = confidential;
    }

    Json(serde_json::json!({
        "success": true,
        "privacy_mode": cfg.privacy.mode,
        "confidential_mode": cfg.privacy.confidential_mode,
    }))
    .into_response()
}

async fn get_privacy_policies_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let rules = state.app.policy_engine.list_rules();
    let ver = state.app.policy_engine.current_version();
    Json(serde_json::json!({
        "version": ver,
        "rules_count": rules.len(),
        "rules": rules
    }))
}

#[derive(Deserialize)]
struct PrivacyEvaluatePayload {
    prompt: String,
    requested_mode: Option<String>,
    provider_id: Option<String>,
    model_id: Option<String>,
    workspace_classification: Option<String>,
}

async fn evaluate_privacy_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<PrivacyEvaluatePayload>,
) -> impl IntoResponse {
    let req_id = RequestId::new();
    let mode = match payload
        .requested_mode
        .as_deref()
        .unwrap_or("local")
        .to_lowercase()
        .as_str()
    {
        "cloud" | "online" => ExecutionMode::Cloud,
        "onpremise" | "on_premise" => ExecutionMode::OnPremise,
        _ => ExecutionMode::Local,
    };
    let ws_class = payload
        .workspace_classification
        .as_deref()
        .unwrap_or("PUBLIC")
        .parse()
        .unwrap_or(octrex_core::privacy::PrivacyClassification::Public);

    let mut ctx = octrex_core::privacy::PrivacyContext::new(req_id);
    ctx.requested_mode = mode;
    ctx.workspace_classification = ws_class;
    ctx.candidate_provider = payload.provider_id;
    ctx.candidate_model = payload.model_id;
    ctx.inputs.push(octrex_core::privacy::PrivacyInput::new(
        octrex_core::privacy::InputSourceType::UserPrompt,
        payload.prompt,
    ));

    let decision = state.app.privacy_gate.evaluate(&ctx);
    Json(serde_json::json!({
        "success": true,
        "decision": decision
    }))
}

#[derive(Deserialize)]
struct ConsentSubmitPayload {
    consent_id: String,
    granted: bool,
    reason: Option<String>,
}

async fn submit_privacy_consent_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ConsentSubmitPayload>,
) -> impl IntoResponse {
    match state.app.privacy_gate.consent_manager().process_decision(
        &payload.consent_id,
        payload.granted,
        payload.reason,
    ) {
        Ok(dec) => {
            Json(serde_json::json!({ "success": true, "consent_decision": dec })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_state_creation() {
        let app_state = ApplicationState::initialize();
        let state = ServerState {
            app: app_state,
            start_time: Instant::now(),
        };
        assert!(state.app.lifecycle.is_ready());
    }
}
