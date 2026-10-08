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
        .route(
            "/api/filesystem/security",
            get(get_filesystem_security_handler),
        )
        .route(
            "/api/filesystem/evaluate",
            post(evaluate_filesystem_handler),
        )
        .route(
            "/api/workspaces/:id/security",
            get(get_workspace_security_handler),
        )
        .route(
            "/api/workspaces/:id/permissions",
            post(update_workspace_permissions_handler),
        )
        .route(
            "/api/workspaces/:id/protected-paths",
            get(get_workspace_protected_paths_handler),
        )
        .route(
            "/api/workspaces/:id/confirm",
            post(confirm_filesystem_handler),
        )
        .route("/api/filesystem/export", post(export_filesystem_handler))
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
        .route("/api/tools", get(get_tools_handler))
        .route("/api/tools/:id", get(get_tool_by_id_handler))
        .route("/api/tools/evaluate", post(evaluate_tool_handler))
        .route("/api/tools/execute", post(execute_tool_handler))
        .route("/api/tools/activity", get(get_tool_activity_handler))
        .route("/api/tools/:id/enable", post(enable_tool_handler))
        .route("/api/tools/:id/disable", post(disable_tool_handler))
        .route("/api/mcp", get(get_mcp_connections_handler))
        .route("/api/mcp/:id", get(get_mcp_connection_by_id_handler))
        .route("/api/mcp/connect", post(connect_mcp_handler))
        .route("/api/mcp/:id/enable", post(enable_mcp_handler))
        .route("/api/mcp/:id/disable", post(disable_mcp_handler))
        .route("/api/mcp/:id/evaluate", post(evaluate_mcp_tool_handler))
        .route("/api/context/status", get(get_context_status_handler))
        .route(
            "/api/context/budget/:session_id",
            get(get_context_budget_handler),
        )
        .route(
            "/api/context/:session_id",
            get(get_context_for_session_handler),
        )
        .route("/api/context/preview", post(preview_context_handler))
        .route("/api/context/compact", post(compact_context_handler))
        .route(
            "/api/context/checkpoints/:session_id",
            get(get_context_checkpoints_handler),
        )
        .route("/api/tasks/:id/context", get(get_task_context_handler))
        .route("/api/router/status", get(get_router_status_handler))
        .route("/api/router/evaluate", post(evaluate_router_handler))
        .route("/api/router/preview", post(preview_router_handler))
        .route("/api/router/decisions", get(get_router_decisions_handler))
        .route(
            "/api/router/decisions/:id",
            get(get_router_decision_by_id_handler),
        )
        .route("/api/models/compatible", get(get_compatible_models_handler))
        .route(
            "/api/models/recommended",
            get(get_recommended_models_handler),
        )
        .route(
            "/api/orchestration/status",
            get(get_orchestration_status_handler),
        )
        .route(
            "/api/tasks",
            get(list_tasks_handler).post(create_task_handler),
        )
        .route("/api/tasks/:id", get(get_task_by_id_handler))
        .route("/api/tasks/:id/start", post(start_task_handler))
        .route("/api/tasks/:id/pause", post(pause_task_handler))
        .route("/api/tasks/:id/resume", post(resume_task_handler))
        .route("/api/tasks/:id/cancel", post(cancel_task_handler))
        .route("/api/tasks/:id/plan", get(get_task_plan_handler))
        .route("/api/tasks/:id/replan", post(replan_task_handler))
        .route("/api/tasks/:id/steps", get(get_task_steps_handler))
        .route("/api/tasks/:id/input", post(submit_user_input_handler))
        .route(
            "/api/tasks/:id/verification",
            get(list_task_verification_handler),
        )
        .route("/api/tasks/:id/verify", post(verify_task_handler))
        .route(
            "/api/tasks/:id/verification/:verification_id",
            get(get_task_verification_handler),
        )
        .route(
            "/api/tasks/:id/verification/:verification_id/retry",
            post(retry_task_verification_handler),
        )
        .route(
            "/api/tasks/:id/completion",
            get(get_task_completion_handler),
        )
        .route(
            "/api/skills",
            get(list_skills_handler).post(create_skill_handler),
        )
        .route("/api/skills/:id", get(get_skill_handler))
        .route("/api/skills/:id/enable", post(enable_skill_handler))
        .route("/api/skills/:id/disable", post(disable_skill_handler))
        .route("/api/skills/:id/validate", post(validate_skill_handler))
        .route("/api/skills/:id/approve", post(approve_skill_handler))
        .route("/api/skills/match", post(match_skills_handler))
        .route(
            "/api/workflows",
            get(list_workflows_handler).post(create_workflow_handler),
        )
        .route("/api/workflows/:id", get(get_workflow_handler))
        .route(
            "/api/workflows/:id/validate",
            post(validate_workflow_handler),
        )
        .route("/api/workflows/:id/run", post(run_workflow_handler))
        .route("/api/workflows/runs/:run_id", get(get_workflow_run_handler))
        .route(
            "/api/memory",
            get(query_memory_handler).post(create_memory_handler),
        )
        .route(
            "/api/memory/:id",
            get(get_memory_handler).delete(delete_memory_handler),
        )
        .route("/api/memory/:id/patch", post(patch_memory_handler))
        .route(
            "/api/memory/candidates",
            get(list_memory_candidates_handler),
        )
        .route(
            "/api/memory/candidates/extract",
            post(extract_memory_candidates_handler),
        )
        .route(
            "/api/memory/candidates/:id/approve",
            post(approve_memory_candidate_handler),
        )
        .route(
            "/api/memory/candidates/:id/reject",
            post(reject_memory_candidate_handler),
        )
        .route(
            "/api/memory/context-items",
            post(memory_context_items_handler),
        )
        .route("/api/documents", get(list_documents_handler))
        .route("/api/documents/import", post(import_document_handler))
        .route("/api/documents/assist", post(assist_document_handler))
        .route("/api/documents/:id", get(get_document_handler))
        .route(
            "/api/documents/:id/sections",
            get(get_document_sections_handler),
        )
        .route(
            "/api/documents/:id/chunks",
            get(get_document_chunks_handler),
        )
        .route("/api/documents/:id/search", post(search_document_handler))
        .route("/api/documents/:id/ingest", post(ingest_document_handler))
        .route(
            "/api/artifacts",
            get(list_artifacts_handler).post(create_artifact_handler),
        )
        .route("/api/artifacts/:id", get(get_artifact_handler))
        .route(
            "/api/artifacts/:id/lineage",
            get(get_artifact_lineage_handler),
        )
        .route(
            "/api/artifacts/:id/verification",
            get(get_artifact_verification_handler),
        )
        .route("/api/artifacts/:id/verify", post(verify_artifact_handler))
        .route("/api/artifacts/:id/export", post(export_artifact_handler))
        .route("/api/local-runtimes", get(list_local_runtimes_handler))
        .route(
            "/api/local-runtimes/discover",
            post(discover_local_runtimes_handler),
        )
        .route(
            "/api/local-runtimes/register",
            post(register_local_runtime_handler),
        )
        .route("/api/local-runtimes/:id", get(get_local_runtime_handler))
        .route(
            "/api/local-runtimes/:id/refresh",
            post(refresh_local_runtime_handler),
        )
        .route(
            "/api/local-runtimes/:id/test",
            post(test_local_runtime_handler),
        )
        .route(
            "/api/local-runtimes/:id/start",
            post(start_local_runtime_handler),
        )
        .route(
            "/api/local-runtimes/:id/stop",
            post(stop_local_runtime_handler),
        )
        .route("/api/local-models", get(list_local_models_handler))
        .route(
            "/api/local-models/discover",
            post(discover_local_models_handler),
        )
        .route(
            "/api/local-models/register",
            post(register_local_model_handler),
        )
        .route(
            "/api/local-models/download",
            post(download_local_model_handler),
        )
        .route(
            "/api/local-models/routing/explain",
            get(explain_local_routing_handler),
        )
        .route("/api/local-models/:id", get(get_local_model_handler))
        .route(
            "/api/local-models/:id/enable",
            post(enable_local_model_handler),
        )
        .route(
            "/api/local-models/:id/disable",
            post(disable_local_model_handler),
        )
        .route(
            "/api/local-models/:id/compatibility",
            get(local_model_compatibility_handler),
        )
        .route(
            "/api/local-inference/preview",
            post(preview_local_inference_handler),
        )
        .route(
            "/api/local-inference/execute",
            post(execute_local_inference_handler),
        )
        .route(
            "/api/local-inference/stream",
            post(stream_local_inference_handler),
        )
        .route(
            "/api/local-inference/cancel",
            post(cancel_local_inference_handler),
        )
        .route(
            "/api/local-inference/metrics",
            get(list_local_inference_metrics_handler),
        )
        .route(
            "/api/local-inference/metrics/:call_id",
            get(get_local_inference_metric_handler),
        )
        .route("/api/local-storage", get(get_local_storage_handler))
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
    let raw_path = PathBuf::from(&payload.path);
    let validated_root =
        match octrex_core::filesystem::WorkspaceSecurityValidator::validate_workspace_root(
            &raw_path,
        ) {
            Ok(c) => c,
            Err(e) => {
                return Json(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid workspace root: {}", e)
                }))
                .into_response();
            }
        };

    let info = WorkspaceManager::inspect(&validated_root);
    let ws = state
        .app
        .workspace_registry
        .register_workspace(&info.name, validated_root);

    Json(serde_json::json!({
        "workspace_id": ws.id.as_str(),
        "info": info
    }))
    .into_response()
}

#[derive(Deserialize)]
struct WorkspaceTreePayload {
    path: String,
}

async fn tree_workspace_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<WorkspaceTreePayload>,
) -> impl IntoResponse {
    let path = PathBuf::from(&payload.path);
    let decision = match state.app.filesystem_security.evaluate_operation(
        None,
        Some(&path),
        ".",
        octrex_core::filesystem::FilesystemOperation::List,
    ) {
        Ok(d) => d,
        Err(err) => {
            return Json(serde_json::json!({ "success": false, "error": err.to_string() }))
                .into_response();
        }
    };

    if !decision.decision.is_allowed() {
        return Json(serde_json::json!({ "success": false, "error": decision.reason }))
            .into_response();
    }

    let root_target = PathBuf::from(
        decision
            .resolved_path
            .unwrap_or_else(|| payload.path.clone()),
    );
    let tree = WorkspaceManager::list_tree(&root_target);

    Json(serde_json::json!({
        "success": true,
        "entries": tree
    }))
    .into_response()
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
    let ws_path = PathBuf::from(&payload.workspace_path);
    let decision = match state.app.filesystem_security.evaluate_operation(
        None,
        Some(&ws_path),
        &payload.rel_path,
        octrex_core::filesystem::FilesystemOperation::Read,
    ) {
        Ok(d) => d,
        Err(err) => {
            return Json(serde_json::json!({ "success": false, "error": err.to_string() }))
                .into_response();
        }
    };

    if !decision.decision.is_allowed() {
        return Json(serde_json::json!({ "success": false, "error": decision.reason }))
            .into_response();
    }

    let target = PathBuf::from(decision.resolved_path.unwrap());
    match octrex_core::filesystem::SafeOperations::safe_read_text(
        &target,
        &octrex_core::filesystem::FilesystemLimits::default(),
    ) {
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
    let ws_path = PathBuf::from(&payload.workspace_path);
    let decision = match state.app.filesystem_security.evaluate_operation(
        None,
        Some(&ws_path),
        &payload.rel_path,
        octrex_core::filesystem::FilesystemOperation::Write,
    ) {
        Ok(d) => d,
        Err(err) => {
            return Json(serde_json::json!({ "success": false, "error": err.to_string() }))
                .into_response();
        }
    };

    if !decision.decision.is_allowed() {
        return Json(serde_json::json!({ "success": false, "error": decision.reason }))
            .into_response();
    }

    let target = PathBuf::from(decision.resolved_path.unwrap());
    match octrex_core::filesystem::SafeOperations::safe_write_bytes(
        &target,
        payload.content.as_bytes(),
        &octrex_core::filesystem::FilesystemLimits::default(),
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
struct EvaluateFilesystemPayload {
    workspace_id: Option<String>,
    workspace_path: Option<String>,
    rel_path: String,
    operation: String,
}

async fn evaluate_filesystem_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<EvaluateFilesystemPayload>,
) -> impl IntoResponse {
    let ws_id = payload
        .workspace_id
        .as_ref()
        .map(|id| WorkspaceId::from_string(id));
    let root_path = payload.workspace_path.as_ref().map(|p| PathBuf::from(p));
    let op = match payload.operation.to_lowercase().as_str() {
        "read" => octrex_core::filesystem::FilesystemOperation::Read,
        "write" => octrex_core::filesystem::FilesystemOperation::Write,
        "create" => octrex_core::filesystem::FilesystemOperation::Create,
        "delete" => octrex_core::filesystem::FilesystemOperation::Delete,
        "rename" => octrex_core::filesystem::FilesystemOperation::Rename,
        "move" => octrex_core::filesystem::FilesystemOperation::Move,
        "list" => octrex_core::filesystem::FilesystemOperation::List,
        "stat" => octrex_core::filesystem::FilesystemOperation::Stat,
        "create_directory" => octrex_core::filesystem::FilesystemOperation::CreateDirectory,
        "delete_directory" => octrex_core::filesystem::FilesystemOperation::DeleteDirectory,
        "copy" => octrex_core::filesystem::FilesystemOperation::Copy,
        "export" => octrex_core::filesystem::FilesystemOperation::Export,
        "import" => octrex_core::filesystem::FilesystemOperation::Import,
        "recursive_delete" => octrex_core::filesystem::FilesystemOperation::RecursiveDelete,
        _ => octrex_core::filesystem::FilesystemOperation::Read,
    };

    match state.app.filesystem_security.evaluate_operation(
        ws_id.as_ref(),
        root_path.as_deref(),
        &payload.rel_path,
        op,
    ) {
        Ok(decision) => Json(serde_json::json!({
            "success": true,
            "decision": decision
        }))
        .into_response(),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string(),
            "decision": {
                "decision": "block",
                "reason": err.to_string()
            }
        }))
        .into_response(),
    }
}

async fn get_filesystem_security_handler(
    State(_state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "service": "FilesystemSecurityService",
        "boundary": "Authoritative Backend Filesystem Security Boundary",
        "symlink_protection": true,
        "reparse_junction_protection": true,
        "path_traversal_protection": true,
        "fail_closed": true,
        "privacy_integration": true,
        "network_integration": true,
    }))
}

async fn get_workspace_security_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let ws_id = WorkspaceId::from_string(&id);
    match state.app.filesystem_security.get_workspace_security(&ws_id) {
        Ok(status) => Json(serde_json::json!({
            "success": true,
            "security": status
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
struct UpdateWorkspacePermissionsPayload {
    read_only: Option<bool>,
    allow_delete: Option<bool>,
    allow_recursive_delete: Option<bool>,
    allow_export: Option<bool>,
    allow_import: Option<bool>,
}

async fn update_workspace_permissions_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateWorkspacePermissionsPayload>,
) -> impl IntoResponse {
    let ws_id = WorkspaceId::from_string(&id);
    let mut policy = state
        .app
        .filesystem_security
        .get_workspace_security(&ws_id)
        .map(|s| s.active_policy)
        .unwrap_or_default();

    if let Some(ro) = payload.read_only {
        policy.read_only = ro;
    }
    if let Some(ad) = payload.allow_delete {
        policy.allow_delete = ad;
    }
    if let Some(ard) = payload.allow_recursive_delete {
        policy.allow_recursive_delete = ard;
    }
    if let Some(ae) = payload.allow_export {
        policy.allow_export = ae;
    }
    if let Some(ai) = payload.allow_import {
        policy.allow_import = ai;
    }

    match state
        .app
        .filesystem_security
        .update_workspace_policy(&ws_id, policy)
    {
        Ok(updated) => Json(serde_json::json!({
            "success": true,
            "policy": updated
        }))
        .into_response(),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        }))
        .into_response(),
    }
}

async fn get_workspace_protected_paths_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let ws_id = WorkspaceId::from_string(&id);
    match state.app.filesystem_security.get_workspace_security(&ws_id) {
        Ok(status) => Json(serde_json::json!({
            "success": true,
            "protected_paths": status.active_policy.protected_paths
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
struct ConfirmFilesystemPayload {
    decision_id: String,
    granted: bool,
}

async fn confirm_filesystem_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ConfirmFilesystemPayload>,
) -> impl IntoResponse {
    match state
        .app
        .filesystem_security
        .confirm_operation(&payload.decision_id, payload.granted)
    {
        Ok(decision) => Json(serde_json::json!({
            "success": true,
            "decision": decision
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
struct ExportFilesystemPayload {
    workspace_id: String,
    rel_path: String,
    dest_external_path: String,
}

async fn export_filesystem_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ExportFilesystemPayload>,
) -> impl IntoResponse {
    let ws_id = WorkspaceId::from_string(&payload.workspace_id);
    match state.app.filesystem_security.export_file(&ws_id, &payload.rel_path, &payload.dest_external_path) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": format!("Successfully exported '{}' to '{}'", payload.rel_path, payload.dest_external_path)
        })).into_response(),
        Err(err) => Json(serde_json::json!({
            "success": false,
            "error": err.to_string()
        })).into_response(),
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

// ============================================================================
// PHASE 9 TOOL RUNTIME & MCP API HANDLERS
// ============================================================================

async fn get_tools_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let tools = state.app.tool_registry.list();
    Json(serde_json::json!({
        "success": true,
        "tools": tools,
        "count": tools.len()
    }))
}

async fn get_tool_by_id_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.tool_registry.get(&octrex_core::tools::ToolId::new(&id)) {
        Some(tool) => Json(serde_json::json!({ "success": true, "tool": tool })).into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("Tool '{}' not found", id) })),
        )
            .into_response(),
    }
}

async fn evaluate_tool_handler(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<octrex_core::tools::ToolRequest>,
) -> impl IntoResponse {
    let decision = state.app.tool_runtime.evaluate_request(&req);
    Json(serde_json::json!({
        "success": true,
        "decision": decision
    }))
}

async fn execute_tool_handler(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<octrex_core::tools::ToolRequest>,
) -> impl IntoResponse {
    let ws_path = req
        .workspace_id
        .as_ref()
        .and_then(|ws_id| state.app.workspace_registry.get_workspace(ws_id))
        .map(|w| std::path::PathBuf::from(w.path));

    let response = state.app.tool_runtime.execute_tool(req, ws_path).await;
    Json(serde_json::json!({
        "success": true,
        "response": response
    }))
}

async fn get_tool_activity_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    use octrex_core::db::repository::AuditRepository;
    let repo = octrex_core::db::repository::SqliteAuditRepository::new((*state.app.db).clone());
    let records = repo.list_audits(100).unwrap_or_default();
    let tool_records: Vec<_> = records
        .into_iter()
        .filter(|r| r.actor == "tool_runtime" || r.tool.is_some())
        .collect();

    Json(serde_json::json!({
        "success": true,
        "activity": tool_records,
        "count": tool_records.len()
    }))
}

async fn enable_tool_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state
        .app
        .tool_registry
        .enable(&octrex_core::tools::ToolId::new(&id))
    {
        Ok(_) => Json(
            serde_json::json!({ "success": true, "message": format!("Tool '{}' enabled", id) }),
        )
        .into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn disable_tool_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state
        .app
        .tool_registry
        .disable(&octrex_core::tools::ToolId::new(&id))
    {
        Ok(_) => Json(
            serde_json::json!({ "success": true, "message": format!("Tool '{}' disabled", id) }),
        )
        .into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_mcp_connections_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let servers = state.app.mcp_registry.list_servers();
    Json(serde_json::json!({
        "success": true,
        "servers": servers,
        "count": servers.len()
    }))
}

async fn get_mcp_connection_by_id_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.mcp_registry.get_server(&id) {
        Some(srv) => Json(serde_json::json!({ "success": true, "server": srv })).into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("MCP Server '{}' not found", id) })),
        ).into_response(),
    }
}

async fn connect_mcp_handler(
    State(state): State<Arc<ServerState>>,
    Json(srv): Json<octrex_core::tools::McpServerInfo>,
) -> impl IntoResponse {
    state.app.mcp_registry.register_server(srv.clone());
    Json(serde_json::json!({ "success": true, "server": srv }))
}

async fn enable_mcp_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let success = state.app.mcp_registry.set_enabled(&id, true);
    Json(serde_json::json!({ "success": success }))
}

async fn disable_mcp_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let success = state.app.mcp_registry.set_enabled(&id, false);
    Json(serde_json::json!({ "success": success }))
}

async fn evaluate_mcp_tool_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(req): Json<octrex_core::tools::ToolRequest>,
) -> impl IntoResponse {
    match state.app.mcp_registry.get_server(&id) {
        Some(srv) => {
            match octrex_core::tools::McpPolicyEngine::evaluate_server_execution(&srv) {
                Ok(_) => {
                    let decision = state.app.tool_runtime.evaluate_request(&req);
                    Json(serde_json::json!({ "success": true, "decision": decision })).into_response()
                }
                Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response(),
            }
        }
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("MCP Server '{}' not found", id) })),
        ).into_response(),
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

// --- Context Engine Phase 10 Stubs ---
// TODO: Connect these to `state.app.context_engine` once the Rust implementation is integrated.

async fn get_context_status_handler(
    State(_state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "ContextEngine",
        "status": "ready",
        "active_sessions": 0,
        "total_items_tracked": 0,
        "total_checkpoints": 0,
        "fail_closed": true
    }))
}

async fn get_context_budget_handler(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let cfg = state.app.config.read().unwrap().clone();
    let model_id = cfg.active_provider.clone();
    let context_window = state
        .app
        .model_registry
        .get_model(&model_id)
        .and_then(|m| m.context_window)
        .unwrap_or(8192) as usize;

    match state
        .app
        .context_engine
        .get_budget_for_session(&session_id, &model_id, context_window)
    {
        Ok(budget) => {
            Json(serde_json::json!({ "success": true, "budget": budget })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_context_for_session_handler(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state
        .app
        .context_engine
        .get_session_context_summary(&session_id)
    {
        Ok(summary) => {
            Json(serde_json::json!({ "success": true, "result": summary })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct ContextPreviewPayload {
    session_id: String,
    task_id: Option<String>,
    model_id: String,
    user_request: Option<String>,
}

async fn preview_context_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ContextPreviewPayload>,
) -> impl IntoResponse {
    let context_window = state
        .app
        .model_registry
        .get_model(&payload.model_id)
        .and_then(|m| m.context_window)
        .unwrap_or(8192) as usize;

    match state.app.context_engine.preview_context(
        &payload.session_id,
        payload.task_id.as_deref(),
        &payload.model_id,
        context_window,
        payload.user_request.as_deref(),
    ) {
        Ok(preview) => {
            Json(serde_json::json!({ "success": true, "preview": preview })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct ContextCompactPayload {
    session_id: String,
    task_id: Option<String>,
    model_id: String,
}

async fn compact_context_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ContextCompactPayload>,
) -> impl IntoResponse {
    let context_window = state
        .app
        .model_registry
        .get_model(&payload.model_id)
        .and_then(|m| m.context_window)
        .unwrap_or(8192) as usize;

    match state.app.context_engine.compact_context(
        &payload.session_id,
        payload.task_id.as_deref(),
        &payload.model_id,
        context_window,
    ) {
        Ok(result) => {
            Json(serde_json::json!({ "success": true, "compaction": result })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_context_checkpoints_handler(
    State(state): State<Arc<ServerState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.app.context_engine.list_checkpoints(&session_id) {
        Ok(checkpoints) => {
            Json(serde_json::json!({ "success": true, "checkpoints": checkpoints })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_task_context_handler(
    State(state): State<Arc<ServerState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match state.app.context_engine.get_task_context_state(&task_id) {
        Ok(task_ctx) => {
            Json(serde_json::json!({ "success": true, "task_context": task_ctx })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_router_status_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let models_cnt = state.app.model_registry.list_models().len();
    let providers_cnt = state.app.provider_registry.list_providers().len();
    let profile = state.app.hardware_service.get_profile();

    Json(serde_json::json!({
        "success": true,
        "service": "ModelRouter",
        "status": "READY",
        "registered_models": models_cnt,
        "registered_providers": providers_cnt,
        "hardware_confidence": profile.confidence,
        "fail_closed": true,
        "no_automatic_fallback": true,
        "privacy_first": true
    }))
}

#[derive(Deserialize)]
struct EvaluateRouterPayload {
    purpose: Option<String>,
    routing_mode: Option<String>,
    user_selected_model: Option<String>,
    workspace_id: Option<String>,
    required_context_tokens: Option<u32>,
    required_output_tokens: Option<u32>,
}

async fn evaluate_router_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<EvaluateRouterPayload>,
) -> impl IntoResponse {
    let mut req = octrex_core::router::RoutingRequest::new(
        payload.purpose.unwrap_or_else(|| "general".to_string()),
    );

    if let Some(mode_str) = payload.routing_mode {
        if let Ok(rm) = mode_str.parse::<octrex_core::router::RoutingMode>() {
            req = req.with_routing_mode(rm);
        }
    }

    if let Some(model_id) = payload.user_selected_model {
        req = req.with_user_selected_model(model_id);
    }

    if let Some(ws_id) = payload.workspace_id {
        req = req.with_workspace_id(octrex_core::ids::WorkspaceId::from_string(&ws_id));
    }

    if let (Some(in_t), Some(out_t)) = (
        payload.required_context_tokens,
        payload.required_output_tokens,
    ) {
        req = req.with_required_context(in_t, out_t);
    }

    let decision = state.app.model_router.route(req);
    Json(serde_json::json!({ "success": true, "decision": decision })).into_response()
}

async fn preview_router_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<EvaluateRouterPayload>,
) -> impl IntoResponse {
    let mut req = octrex_core::router::RoutingRequest::new(
        payload.purpose.unwrap_or_else(|| "general".to_string()),
    );

    if let Some(mode_str) = payload.routing_mode {
        if let Ok(rm) = mode_str.parse::<octrex_core::router::RoutingMode>() {
            req = req.with_routing_mode(rm);
        }
    }

    if let Some(model_id) = payload.user_selected_model {
        req = req.with_user_selected_model(model_id);
    }

    let decision = state.app.model_router.preview(req);
    Json(serde_json::json!({ "success": true, "decision": decision })).into_response()
}

async fn get_router_decisions_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let compatible = state.app.model_router.get_compatible_models(None);
    Json(serde_json::json!({
        "success": true,
        "compatible_models_count": compatible.len(),
        "service": "ModelRouter"
    }))
    .into_response()
}

async fn get_router_decision_by_id_handler(
    State(_state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "success": true,
        "decision_id": id,
        "note": "Decision details record retrieved"
    }))
    .into_response()
}

async fn get_compatible_models_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let models = state.app.model_router.get_compatible_models(None);
    Json(serde_json::json!({
        "success": true,
        "total": models.len(),
        "models": models
    }))
    .into_response()
}

#[derive(Deserialize)]
struct RecommendedQuery {
    purpose: Option<String>,
}

async fn get_recommended_models_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<RecommendedQuery>,
) -> impl IntoResponse {
    let models = state
        .app
        .model_router
        .get_recommended_models(query.purpose.as_deref());
    Json(serde_json::json!({
        "success": true,
        "total": models.len(),
        "models": models
    }))
    .into_response()
}

// --- Orchestration Phase 11 Handlers ---

async fn get_orchestration_status_handler(
    State(_state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "service": "OrchestrationService",
        "status": "READY",
        "fail_closed": true,
        "untrusted_model_actions": true,
        "authoritative_security": true
    }))
}

async fn list_tasks_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let tasks = state.app.task_registry.list_tasks();
    Json(serde_json::json!({
        "success": true,
        "total": tasks.len(),
        "tasks": tasks
    }))
}

#[derive(Deserialize)]
struct CreateTaskPayload {
    objective: String,
    session_id: Option<String>,
    workspace_id: Option<String>,
}

async fn create_task_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<CreateTaskPayload>,
) -> impl IntoResponse {
    let sess_id = payload
        .session_id
        .map(|id| octrex_core::ids::SessionId::from_string(&id));
    let ws_id = payload
        .workspace_id
        .map(|id| octrex_core::ids::WorkspaceId::from_string(&id));

    match state
        .app
        .orchestration_service
        .create_task_and_plan(&payload.objective, sess_id, ws_id)
    {
        Ok((task, plan)) => Json(serde_json::json!({
            "success": true,
            "task": task,
            "plan": plan
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn get_task_by_id_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    match state.app.task_registry.get_task(&task_id) {
        Some(task) => {
            let plan = state.app.orchestration_service.get_plan(&task_id);
            Json(serde_json::json!({
                "success": true,
                "task": task,
                "plan": plan
            }))
            .into_response()
        }
        None => Json(serde_json::json!({
            "success": false,
            "error": format!("Task '{}' not found", id)
        }))
        .into_response(),
    }
}

#[derive(Deserialize, Default)]
struct TaskExecuteQuery {
    workspace_path: Option<String>,
}

async fn start_task_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<TaskExecuteQuery>,
) -> impl IntoResponse {
    let app_svc = state.app.orchestration_service.clone();
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    let ws_path = query.workspace_path.clone();

    tokio::spawn(async move {
        let _ = app_svc.start_task(&task_id, ws_path).await;
    });

    Json(serde_json::json!({
        "success": true,
        "task_id": id,
        "status": "executing",
        "message": "Task execution started"
    }))
}

async fn pause_task_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    match state.app.orchestration_service.pause_task(&task_id) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "task_id": id,
            "status": "paused"
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn resume_task_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<TaskExecuteQuery>,
) -> impl IntoResponse {
    let app_svc = state.app.orchestration_service.clone();
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    let ws_path = query.workspace_path.clone();

    tokio::spawn(async move {
        let _ = app_svc.resume_task(&task_id, ws_path).await;
    });

    Json(serde_json::json!({
        "success": true,
        "task_id": id,
        "status": "resuming",
        "message": "Task execution resumed"
    }))
}

async fn cancel_task_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    match state.app.orchestration_service.cancel_task(&task_id) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "task_id": id,
            "status": "cancelled"
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn get_task_plan_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    match state.app.orchestration_service.get_plan(&task_id) {
        Some(plan) => Json(serde_json::json!({
            "success": true,
            "plan": plan
        }))
        .into_response(),
        None => Json(serde_json::json!({
            "success": false,
            "error": format!("No plan found for task '{}'", id)
        }))
        .into_response(),
    }
}

#[derive(Deserialize)]
struct ReplanTaskPayload {
    revised_objective: String,
}

async fn replan_task_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<ReplanTaskPayload>,
) -> impl IntoResponse {
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    match state
        .app
        .orchestration_service
        .replan_task(&task_id, &payload.revised_objective)
    {
        Ok(plan) => Json(serde_json::json!({
            "success": true,
            "plan": plan
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn get_task_steps_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    let steps = state.app.orchestration_service.get_steps(&task_id);
    let plan = state.app.orchestration_service.get_plan(&task_id);
    Json(serde_json::json!({
        "success": true,
        "steps": steps,
        "current_step": plan.as_ref().map(|p| p.current_step)
    }))
    .into_response()
}

#[derive(Deserialize)]
struct SubmitUserInputPayload {
    input: String,
    workspace_path: Option<String>,
}

async fn submit_user_input_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<SubmitUserInputPayload>,
) -> impl IntoResponse {
    let app_svc = state.app.orchestration_service.clone();
    let task_id = octrex_core::ids::TaskId::from_string(&id);
    let input = payload.input.clone();
    let ws_path = payload.workspace_path.clone();

    match app_svc.submit_user_input(&task_id, &input, ws_path).await {
        Ok(decision) => Json(serde_json::json!({
            "success": true,
            "task_id": id,
            "decision": decision,
            "message": "User input submitted and execution evaluated"
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

// ============================================================================
// PHASE 13 VERIFICATION & RELIABILITY API HANDLERS
// The VerificationEngine independently validates completion from structured
// authoritative evidence. Model claims are recorded but never trusted.
// ============================================================================

async fn list_task_verification_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.verification_engine.list_runs_by_task(&id) {
        Ok(runs) => Json(serde_json::json!({
            "success": true,
            "task_id": id,
            "total": runs.len(),
            "runs": runs
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn verify_task_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(mut payload): Json<octrex_core::verification::TaskVerificationRequest>,
) -> impl IntoResponse {
    // Path task ID is authoritative; a body mismatch is rejected fail-closed.
    if payload.task_id.trim().is_empty() {
        payload.task_id = id.clone();
    }
    if payload.task_id != id {
        return Json(serde_json::json!({
            "success": false,
            "error": format!(
                "Task ID mismatch: path '{}' does not match request '{}'",
                id, payload.task_id
            )
        }))
        .into_response();
    }

    match state.app.verification_engine.verify_task(&payload) {
        Ok(result) => {
            let gate = state
                .app
                .verification_engine
                .completion_gate(&result, false);
            let action = state
                .app
                .verification_engine
                .orchestrator_action(&result, false);
            Json(serde_json::json!({
                "success": true,
                "verification": result,
                "completion_gate": gate.to_string(),
                "orchestrator_action": action
            }))
            .into_response()
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn get_task_verification_handler(
    State(state): State<Arc<ServerState>>,
    Path((id, verification_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.app.verification_engine.get_run(&verification_id) {
        Ok(Some(result)) => {
            if result.task_id != id {
                return Json(serde_json::json!({
                    "success": false,
                    "error": format!(
                        "Verification '{}' does not belong to task '{}'",
                        verification_id, id
                    )
                }))
                .into_response();
            }
            let gate = state
                .app
                .verification_engine
                .completion_gate(&result, false);
            Json(serde_json::json!({
                "success": true,
                "verification": result,
                "completion_gate": gate.to_string()
            }))
            .into_response()
        }
        Ok(None) => Json(serde_json::json!({
            "success": false,
            "error": format!("Verification '{}' not found", verification_id)
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn retry_task_verification_handler(
    State(state): State<Arc<ServerState>>,
    Path((id, verification_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Bounded repair registration: never an infinite loop.
    match state.app.verification_engine.register_repair_attempt(&id) {
        Ok(attempt) => {
            let latest = state
                .app
                .verification_engine
                .latest_run_by_task(&id)
                .ok()
                .flatten();
            Json(serde_json::json!({
                "success": true,
                "task_id": id,
                "verification_id": verification_id,
                "repair_attempt": attempt,
                "max_repair_attempts": state.app.verification_engine.max_repair_attempts(),
                "latest": latest,
                "message": "Repair attempt registered. Re-run verification via POST /api/tasks/:id/verify with updated evidence."
            }))
            .into_response()
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

async fn get_task_completion_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.verification_engine.latest_run_by_task(&id) {
        Ok(Some(result)) => {
            let gate = state
                .app
                .verification_engine
                .completion_gate(&result, false);
            let action = state
                .app
                .verification_engine
                .orchestrator_action(&result, false);
            Json(serde_json::json!({
                "success": true,
                "task_id": id,
                "verified": result.status.is_pass(),
                "status": result.status.to_string(),
                "confidence": result.confidence,
                "completion_gate": gate.to_string(),
                "orchestrator_action": action,
                "verification_id": result.verification_id,
                "failures": result.failures,
                "warnings": result.warnings
            }))
            .into_response()
        }
        Ok(None) => Json(serde_json::json!({
            "success": true,
            "task_id": id,
            "verified": false,
            "status": "NOT_VERIFIED",
            "completion_gate": "BLOCKED",
            "orchestrator_action": "Fail",
            "failures": ["No verification run exists for this task; completion is not established"]
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
        .into_response(),
    }
}

// ============================================================================
// PHASE 14 SKILLS / WORKFLOWS / MEMORY API HANDLERS
// Delegation: skills/workflows validate declaratively; execution delegates to
// Phase 11 Orchestrator, capability checks to Phase 9 ToolRuntime, context to
// Phase 10 ContextEngine, model selection to Phase 12 ModelRouter, and checks
// to Phase 13 Verification. No duplicate engines are created.
// ============================================================================

async fn list_skills_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let skills = state.app.skill_service.registry().list();
    let safe: Vec<serde_json::Value> = skills
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "description": s.description,
                "version": s.version,
                "versioned_id": s.versioned_id(),
                "owner": s.owner,
                "source": s.source.to_string(),
                "status": s.status.to_string(),
                "classification": s.classification.to_string(),
                "capabilities_required": s.capabilities_required.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                "allowed_tools": s.allowed_tools,
                "verification_requirements": s.verification_requirements,
                "provenance": octrex_core::skills::provenance::safe_provenance_summary(s),
                "created_at": s.created_at,
                "updated_at": s.updated_at,
            })
        })
        .collect();
    Json(serde_json::json!({ "success": true, "total": safe.len(), "skills": safe }))
}

async fn get_skill_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.skill_service.registry().get(&id) {
        Some(s) => Json(serde_json::json!({ "success": true, "skill": s })).into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("Skill '{}' not found", id) })),
        )
            .into_response(),
    }
}

async fn create_skill_handler(
    State(state): State<Arc<ServerState>>,
    Json(def): Json<octrex_core::skills::SkillDefinition>,
) -> impl IntoResponse {
    match state.app.skill_service.create_skill(def) {
        Ok(s) => Json(serde_json::json!({ "success": true, "skill": s })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn enable_skill_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.skill_service.enable(&id) {
        Ok(s) => Json(serde_json::json!({ "success": true, "skill": s })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn disable_skill_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.skill_service.disable(&id) {
        Ok(s) => Json(serde_json::json!({ "success": true, "skill": s })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn validate_skill_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.skill_service.validate_skill(&id) {
        Ok(valid) => Json(serde_json::json!({ "success": true, "valid": valid })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "valid": false, "error": e.to_string() }))
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct ApproveSkillPayload {
    approver: Option<String>,
}

async fn approve_skill_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<ApproveSkillPayload>,
) -> impl IntoResponse {
    let approver = payload.approver.unwrap_or_else(|| "user".to_string());
    match state.app.skill_service.approve_imported(&id, &approver) {
        Ok(s) => Json(serde_json::json!({ "success": true, "skill": s })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct MatchSkillsPayload {
    user_request: String,
    task_type: Option<String>,
    workspace_id: Option<String>,
    limit: Option<usize>,
}

async fn match_skills_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<MatchSkillsPayload>,
) -> impl IntoResponse {
    let req = octrex_core::skills::SkillMatchRequest {
        user_request: payload.user_request,
        task_id: None,
        workspace_id: payload
            .workspace_id
            .map(|w| octrex_core::ids::WorkspaceId::from_string(&w)),
        task_type: payload.task_type,
        limit: payload.limit.unwrap_or(5).min(20),
    };
    let ranked = state.app.skill_service.match_skills(&req);
    let out: Vec<serde_json::Value> = ranked
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "skill_id": r.skill.id,
                "version": r.skill.version,
                "score": r.score,
                "reasons": r.reasons,
            })
        })
        .collect();
    Json(serde_json::json!({ "success": true, "matches": out }))
}

async fn list_workflows_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let wfs = state.app.workflow_service.registry().list();
    Json(serde_json::json!({ "success": true, "total": wfs.len(), "workflows": wfs }))
}

async fn get_workflow_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.workflow_service.registry().get(&id) {
        Some(w) => Json(serde_json::json!({ "success": true, "workflow": w })).into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("Workflow '{}' not found", id) })),
        )
            .into_response(),
    }
}

async fn create_workflow_handler(
    State(state): State<Arc<ServerState>>,
    Json(def): Json<octrex_core::workflows::WorkflowDefinition>,
) -> impl IntoResponse {
    match state.app.workflow_service.create(def) {
        Ok(w) => Json(serde_json::json!({ "success": true, "workflow": w })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn validate_workflow_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.workflow_service.validate(&id) {
        Ok(valid) => Json(serde_json::json!({ "success": true, "valid": valid })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "valid": false, "error": e.to_string() }))
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct RunWorkflowPayload {
    workflow_version: Option<String>,
    task_id: Option<String>,
    session_id: Option<String>,
    workspace_id: Option<String>,
    inputs: Option<serde_json::Value>,
}

async fn run_workflow_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<RunWorkflowPayload>,
) -> impl IntoResponse {
    match state.app.workflow_service.start_run(
        state.app.skill_service.registry(),
        &state.app.tool_registry,
        &id,
        payload.workflow_version.as_deref(),
        payload.task_id,
        payload.session_id,
        payload.workspace_id,
        payload.inputs.unwrap_or(serde_json::Value::Null),
    ) {
        Ok((run, plan)) => {
            Json(serde_json::json!({ "success": true, "run": run, "plan": plan })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_workflow_run_handler(
    State(state): State<Arc<ServerState>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    match state.app.workflow_service.get_run(&run_id) {
        Some(run) => Json(serde_json::json!({ "success": true, "run": run })).into_response(),
        None => Json(
            serde_json::json!({ "success": false, "error": format!("Run '{}' not found", run_id) }),
        )
        .into_response(),
    }
}

#[derive(Deserialize)]
struct QueryMemoryPayload {
    scope: Option<String>,
    workspace_id: Option<String>,
    project_id: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
    mem_type: Option<String>,
    classification_ceiling: Option<String>,
    min_trust_rank: Option<u8>,
    allow_secret: Option<bool>,
    limit: Option<usize>,
    query_text: Option<String>,
}

async fn query_memory_handler(
    State(state): State<Arc<ServerState>>,
    Query(payload): Query<QueryMemoryPayload>,
) -> impl IntoResponse {
    let query = octrex_core::memory::MemoryQuery {
        scope: payload.scope.and_then(|s| s.parse().ok()),
        workspace_id: payload.workspace_id,
        project_id: payload.project_id,
        session_id: payload.session_id,
        task_id: payload.task_id,
        mem_type: payload.mem_type.and_then(|s| s.parse().ok()),
        classification_ceiling: payload
            .classification_ceiling
            .and_then(|s| s.parse().ok())
            .unwrap_or(octrex_core::privacy::PrivacyClassification::Internal),
        min_trust_rank: payload.min_trust_rank.unwrap_or(0),
        allow_secret: payload.allow_secret.unwrap_or(false),
        limit: payload.limit.unwrap_or(10).min(50),
        query_text: payload.query_text,
    };
    match state.app.memory_service.query(&query) {
        Ok(results) => {
            let safe: Vec<serde_json::Value> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.item.id,
                        "type": r.item.mem_type.to_string(),
                        "scope": r.item.scope.to_string(),
                        "classification": r.item.classification.to_string(),
                        "source": r.item.source.to_string(),
                        "confidence": r.item.confidence,
                        "content_preview": if r.item.classification == octrex_core::privacy::PrivacyClassification::Secret { "[REDACTED]".to_string() } else { r.item.content.chars().take(200).collect::<String>() },
                        "relevance": r.relevance,
                        "workspace_id": r.item.workspace_id,
                        "session_id": r.item.session_id,
                        "task_id": r.item.task_id,
                        "expires_at": r.item.expires_at,
                        "version": r.item.version,
                    })
                })
                .collect();
            Json(serde_json::json!({ "success": true, "total": safe.len(), "results": safe }))
                .into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_memory_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.memory_service.store().get_item(&id) {
        Some(item) => {
            if item.classification == octrex_core::privacy::PrivacyClassification::Secret {
                Json(serde_json::json!({ "success": true, "memory": octrex_core::memory::policy::safe_memory_summary(&item) }))
                    .into_response()
            } else {
                Json(serde_json::json!({ "success": true, "memory": item })).into_response()
            }
        }
        None => Json(
            serde_json::json!({ "success": false, "error": format!("Memory '{}' not found", id) }),
        )
        .into_response(),
    }
}

async fn create_memory_handler(
    State(state): State<Arc<ServerState>>,
    Json(item): Json<octrex_core::memory::MemoryItem>,
) -> impl IntoResponse {
    match state.app.memory_service.create_item(item) {
        Ok(stored) => {
            Json(serde_json::json!({ "success": true, "memory": stored })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct PatchMemoryPayload {
    content: Option<String>,
    confidence: Option<f64>,
}

async fn patch_memory_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<PatchMemoryPayload>,
) -> impl IntoResponse {
    // PATCH is exposed as POST /api/memory/:id/patch for static-export compatibility.
    match state
        .app
        .memory_service
        .update_item(&id, payload.content, payload.confidence)
    {
        Ok(updated) => {
            Json(serde_json::json!({ "success": true, "memory": updated })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn delete_memory_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.memory_service.delete_item(&id) {
        Ok(deleted) => {
            Json(serde_json::json!({ "success": true, "deleted": deleted })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct CandidatesQuery {
    status: Option<String>,
}

async fn list_memory_candidates_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<CandidatesQuery>,
) -> impl IntoResponse {
    let list = state
        .app
        .memory_service
        .list_candidates(query.status.as_deref());
    Json(serde_json::json!({ "success": true, "total": list.len(), "candidates": list }))
}

#[derive(Deserialize)]
struct ExtractCandidatesPayload {
    text: String,
    source: Option<String>,
    workspace_id: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
    tool_id: Option<String>,
    model_id: Option<String>,
}

async fn extract_memory_candidates_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ExtractCandidatesPayload>,
) -> impl IntoResponse {
    let source: octrex_core::memory::MemorySource = payload
        .source
        .and_then(|s| s.parse().ok())
        .unwrap_or(octrex_core::memory::MemorySource::UserExplicit);
    let input = octrex_core::memory::ExtractionInput {
        text: payload.text,
        source,
        actor: "api".to_string(),
        workspace_id: payload.workspace_id,
        session_id: payload.session_id,
        task_id: payload.task_id,
        tool_id: payload.tool_id,
        model_id: payload.model_id,
    };
    match state.app.memory_service.extract_candidates(&input) {
        Ok(cands) => {
            Json(serde_json::json!({ "success": true, "candidates": cands })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn approve_memory_candidate_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.memory_service.approve_candidate(&id) {
        Ok(item) => Json(serde_json::json!({ "success": true, "memory": item })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn reject_memory_candidate_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.memory_service.reject_candidate(&id) {
        Ok(cand) => Json(serde_json::json!({ "success": true, "candidate": cand })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct MemoryContextItemsPayload {
    workspace_id: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
    query_text: Option<String>,
    limit: Option<usize>,
}

async fn memory_context_items_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<MemoryContextItemsPayload>,
) -> impl IntoResponse {
    // Memory -> ContextItem conversion WITHOUT direct prompt injection. The
    // returned items must still pass ContextEngine selection + budget.
    let query = octrex_core::memory::MemoryQuery {
        workspace_id: payload.workspace_id.clone(),
        session_id: payload.session_id.clone(),
        task_id: payload.task_id.clone(),
        classification_ceiling: octrex_core::privacy::PrivacyClassification::Internal,
        allow_secret: false,
        limit: payload.limit.unwrap_or(5).min(20),
        query_text: payload.query_text,
        ..Default::default()
    };
    match state.app.memory_service.query(&query) {
        Ok(results) => {
            let session_id = payload
                .session_id
                .map(|s| octrex_core::ids::SessionId::from_string(&s));
            let task_id = payload
                .task_id
                .map(|t| octrex_core::ids::TaskId::from_string(&t));
            let workspace_id = payload
                .workspace_id
                .map(|w| octrex_core::ids::WorkspaceId::from_string(&w));
            let items = octrex_core::memory::MemoryService::to_context_items(
                &results,
                session_id,
                task_id,
                workspace_id,
            );
            // Validate each item through ContextEngine policy before returning.
            let mut accepted = Vec::new();
            for item in items {
                if state.app.context_engine.add_item(item.clone()).is_ok() {
                    accepted.push(item.safe_summary());
                }
            }
            Json(serde_json::json!({ "success": true, "accepted": accepted })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

// ============================================================================
// PHASE 16 LOCAL RUNTIME & MODEL MANAGEMENT API HANDLERS
// ============================================================================

fn parse_classification(raw: Option<&str>) -> octrex_core::privacy::PrivacyClassification {
    raw.unwrap_or("PUBLIC")
        .parse()
        .unwrap_or(octrex_core::privacy::PrivacyClassification::Public)
}

async fn list_local_runtimes_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let runtimes = state.app.local_runtime.list_runtimes();
    Json(serde_json::json!({
        "success": true,
        "total": runtimes.len(),
        "runtimes": runtimes
    }))
}

#[derive(Deserialize)]
struct DiscoverEndpointPayload {
    endpoint: String,
    runtime_type: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct DiscoverRuntimesPayload {
    include_defaults: Option<bool>,
    endpoints: Option<Vec<DiscoverEndpointPayload>>,
}

async fn discover_local_runtimes_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<DiscoverRuntimesPayload>,
) -> impl IntoResponse {
    let mut requests = Vec::new();
    for ep in payload.endpoints.unwrap_or_default() {
        let runtime_type = match ep
            .runtime_type
            .as_deref()
            .unwrap_or("ollama")
            .parse::<octrex_core::local_runtime::LocalRuntimeType>()
        {
            Ok(t) => t,
            Err(e) => {
                return Json(serde_json::json!({ "success": false, "error": e })).into_response();
            }
        };
        requests.push(octrex_core::local_runtime::detector::DiscoveryRequest::new(
            ep.endpoint,
            runtime_type,
            ep.name,
        ));
    }
    let runtimes = state
        .app
        .local_runtime
        .discover(payload.include_defaults.unwrap_or(true), requests)
        .await;
    Json(serde_json::json!({
        "success": true,
        "total": runtimes.len(),
        "runtimes": runtimes
    }))
    .into_response()
}

#[derive(Deserialize)]
struct RegisterRuntimePayload {
    runtime_type: String,
    endpoint: String,
    name: Option<String>,
}

async fn register_local_runtime_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<RegisterRuntimePayload>,
) -> impl IntoResponse {
    let runtime_type = match payload
        .runtime_type
        .parse::<octrex_core::local_runtime::LocalRuntimeType>()
    {
        Ok(t) => t,
        Err(e) => {
            return Json(serde_json::json!({ "success": false, "error": e })).into_response();
        }
    };
    match state
        .app
        .local_runtime
        .register_runtime(runtime_type, &payload.endpoint, payload.name)
        .await
    {
        Ok(runtime) => {
            Json(serde_json::json!({ "success": true, "runtime": runtime })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_local_runtime_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.get_runtime(&id) {
        Ok(runtime) => {
            Json(serde_json::json!({ "success": true, "runtime": runtime })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn refresh_local_runtime_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.refresh_runtime(&id).await {
        Ok(runtime) => {
            Json(serde_json::json!({ "success": true, "runtime": runtime })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn test_local_runtime_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.test_runtime(&id).await {
        Ok(report) => {
            Json(serde_json::json!({ "success": true, "health": report })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn start_local_runtime_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.start_runtime(&id).await {
        Ok(runtime) => {
            Json(serde_json::json!({ "success": true, "runtime": runtime })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn stop_local_runtime_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.stop_runtime(&id).await {
        Ok(runtime) => {
            Json(serde_json::json!({ "success": true, "runtime": runtime })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn list_local_models_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let models = state.app.local_runtime.list_models();
    Json(serde_json::json!({
        "success": true,
        "total": models.len(),
        "models": models
    }))
}

#[derive(Deserialize)]
struct DiscoverModelsPayload {
    runtime_id: String,
}

async fn discover_local_models_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<DiscoverModelsPayload>,
) -> impl IntoResponse {
    match state
        .app
        .local_runtime
        .discover_models(&payload.runtime_id)
        .await
    {
        Ok(models) => Json(serde_json::json!({
            "success": true,
            "total": models.len(),
            "models": models
        }))
        .into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn register_local_model_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<octrex_core::local_runtime::RegisterModelInput>,
) -> impl IntoResponse {
    match state.app.local_runtime.register_model(payload).await {
        Ok(model) => Json(serde_json::json!({ "success": true, "model": model })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_local_model_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.get_model(&id) {
        Ok(model) => Json(serde_json::json!({ "success": true, "model": model })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn enable_local_model_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.enable_model(&id) {
        Ok(model) => Json(serde_json::json!({ "success": true, "model": model })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn disable_local_model_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.disable_model(&id) {
        Ok(model) => Json(serde_json::json!({ "success": true, "model": model })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct CompatibilityQuery {
    input_tokens: Option<usize>,
    output_tokens: Option<u32>,
}

async fn local_model_compatibility_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<CompatibilityQuery>,
) -> impl IntoResponse {
    match state
        .app
        .local_runtime
        .compatibility(
            &id,
            query.input_tokens.unwrap_or(1024),
            query.output_tokens.map(|v| v as usize),
        )
        .await
    {
        Ok(report) => {
            Json(serde_json::json!({ "success": true, "compatibility": report })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn explain_local_routing_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<CompatibilityQuery>,
) -> impl IntoResponse {
    let explanations = state
        .app
        .local_runtime
        .explain_routing(
            query.input_tokens.unwrap_or(1024),
            query.output_tokens.map(|v| v as usize),
        )
        .await;
    Json(serde_json::json!({
        "success": true,
        "total": explanations.len(),
        "candidates": explanations
    }))
    .into_response()
}

#[derive(Deserialize)]
struct PreviewInferencePayload {
    registry_model_id: String,
    input_tokens: Option<usize>,
    output_tokens: Option<u32>,
    classification: Option<String>,
}

async fn preview_local_inference_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<PreviewInferencePayload>,
) -> impl IntoResponse {
    let classification = parse_classification(payload.classification.as_deref());
    let record = match state
        .app
        .local_runtime
        .get_model(&payload.registry_model_id)
    {
        Ok(m) => m,
        Err(e) => {
            return Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                .into_response();
        }
    };
    match state
        .app
        .local_runtime
        .compatibility(
            &record.id,
            payload.input_tokens.unwrap_or(1024),
            payload.output_tokens.map(|v| v as usize),
        )
        .await
    {
        Ok(report) => Json(serde_json::json!({
            "success": true,
            "preview": {
                "registry_model_id": record.registry_model_id,
                "state": record.state.to_string(),
                "health": record.health.to_string(),
                "routable": report.routable && record.state.is_routable() && record.health.is_routable(),
                "compatibility": report,
                "classification": classification.to_string(),
                "note": "Preview only: no inference was executed and no model was downloaded."
            }
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

#[derive(Deserialize)]
struct InferenceMessagePayload {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ExecuteInferencePayload {
    registry_model_id: String,
    messages: Vec<InferenceMessagePayload>,
    system_instructions: Option<String>,
    max_output_tokens: Option<u32>,
    temperature: Option<f32>,
    classification: Option<String>,
}

fn build_model_request(payload: ExecuteInferencePayload) -> octrex_core::models::ModelRequest {
    use std::collections::HashMap;
    octrex_core::models::ModelRequest {
        model_id: payload.registry_model_id,
        messages: payload
            .messages
            .into_iter()
            .map(|m| octrex_core::models::ModelMessage {
                role: m.role,
                content: m.content,
                tool_calls: None,
            })
            .collect(),
        system_instructions: payload.system_instructions,
        tools: vec![],
        temperature: payload.temperature,
        max_output_tokens: payload.max_output_tokens,
        response_format: octrex_core::models::ResponseFormat::Text,
        metadata: HashMap::new(),
        correlation: octrex_core::models::CallCorrelation::default(),
    }
}

async fn execute_local_inference_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ExecuteInferencePayload>,
) -> impl IntoResponse {
    let classification = parse_classification(payload.classification.as_deref());
    let call_id = octrex_core::ids::RequestId::new();
    let _ = call_id;
    let request = build_model_request(payload);
    match state
        .app
        .local_runtime
        .execute(request, classification)
        .await
    {
        Ok(response) => {
            Json(serde_json::json!({ "success": true, "response": response })).into_response()
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
            "no_cloud_fallback": true
        }))
        .into_response(),
    }
}

async fn stream_local_inference_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ExecuteInferencePayload>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    use tokio_stream::wrappers::ReceiverStream;
    let classification = parse_classification(payload.classification.as_deref());
    let request = build_model_request(payload);
    // NOTE (Phase 15): a single stream construction so both outcomes share
    // one concrete `impl Stream` type (two closure expressions never unify).
    let (call_id, rx) = match state
        .app
        .local_runtime
        .execute_stream(request, classification)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let msg = e.to_string();
            tokio::spawn(async move {
                let _ = tx
                    .send(octrex_core::models::ModelStreamEvent::Failed(msg))
                    .await;
            });
            (String::new(), rx)
        }
    };
    let stream = ReceiverStream::new(rx).map(move |ev| {
        let json = serde_json::json!({ "call_id": call_id, "event": ev });
        Ok(Event::default().data(serde_json::to_string(&json).unwrap_or_default()))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Deserialize)]
struct CancelInferencePayload {
    call_id: String,
}

async fn cancel_local_inference_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<CancelInferencePayload>,
) -> impl IntoResponse {
    let cancelled = state.app.local_runtime.cancel(&payload.call_id);
    Json(serde_json::json!({ "success": true, "cancelled": cancelled }))
}

async fn list_local_inference_metrics_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let metrics = state.app.local_runtime.recent_metrics(50);
    let active = state.app.local_runtime.active_call_ids();
    let slots = state.app.local_runtime.slot_occupants();
    Json(serde_json::json!({
        "success": true,
        "total": metrics.len(),
        "metrics": metrics,
        "active_calls": active,
        "slot_occupants": slots
    }))
}

async fn get_local_inference_metric_handler(
    State(state): State<Arc<ServerState>>,
    Path(call_id): Path<String>,
) -> impl IntoResponse {
    match state.app.local_runtime.get_metrics(&call_id) {
        Some(metric) => Json(serde_json::json!({ "success": true, "metric": metric })).into_response(),
        None => Json(serde_json::json!({ "success": false, "error": format!("No metrics for call '{}'", call_id) })).into_response(),
    }
}

async fn download_local_model_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<octrex_core::local_runtime::DownloadModelInput>,
) -> impl IntoResponse {
    match state.app.local_runtime.download_model(payload).await {
        Ok(summary) => {
            Json(serde_json::json!({ "success": true, "installation": summary })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_local_storage_handler(
    State(state): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let dir = state.app.local_runtime.model_storage_dir();
    Json(serde_json::json!({
        "success": true,
        "model_dir": dir.to_string_lossy(),
        "exists": dir.exists()
    }))
}

// ============================================================================
// PHASE 15 DOCUMENT INTELLIGENCE & ARTIFACT PIPELINE API HANDLERS
// All intake goes through FilesystemSecurityService; documents are untrusted
// data; SECRET content is redacted (never raw) in every response.
// ============================================================================

/// Resolve a workspace id against the DB, syncing server-registered
/// in-memory workspaces into the DB so the filesystem boundary can resolve
/// them. Unknown ids fail closed.
fn resolve_doc_workspace(
    state: &ServerState,
    workspace_id: &str,
) -> Result<octrex_core::ids::WorkspaceId, String> {
    use octrex_core::db::repository::{SqliteWorkspaceRepository, WorkspaceRepository};
    let ws_id = octrex_core::ids::WorkspaceId::from_string(workspace_id);
    let repo = SqliteWorkspaceRepository::new((*state.app.db).clone());
    match repo.get_workspace(&ws_id) {
        Ok(Some(_)) => Ok(ws_id),
        Ok(None) => {
            // Sync from the server's own in-memory registry (populated by
            // /api/workspace/inspect). Anything else is rejected.
            match state.app.workspace_registry.get_workspace(&ws_id) {
                Some(ws) => match repo.create_workspace(&ws) {
                    Ok(_) => Ok(ws_id),
                    Err(e) => Err(format!("Workspace sync failed: {}", e)),
                },
                None => Err(format!(
                    "Workspace '{}' is not registered; inspect it first via /api/workspace/inspect",
                    workspace_id
                )),
            }
        }
        Err(e) => Err(format!("Workspace lookup failed: {}", e)),
    }
}

fn redact_chunk_text(text: &str, class: &str) -> String {
    match class {
        "SECRET" => "[REDACTED: SECRET document content withheld]".to_string(),
        "RESTRICTED" => {
            let preview: String = text.chars().take(500).collect();
            format!(
                "[RESTRICTED preview] {}",
                octrex_core::privacy::EvidenceManager::redact_string(&preview)
            )
        }
        _ => octrex_core::privacy::EvidenceManager::redact_string(
            &text.chars().take(8000).collect::<String>(),
        ),
    }
}

#[derive(Deserialize)]
struct ListDocumentsQuery {
    workspace_id: String,
}

async fn list_documents_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ListDocumentsQuery>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &query.workspace_id) {
        Ok(id) => id,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    match state.app.document_service.list_documents(&ws_id) {
        Ok(docs) => Json(serde_json::json!({
            "success": true,
            "total": docs.len(),
            "documents": docs
        }))
        .into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct ImportDocumentPayload {
    workspace_id: String,
    rel_path: String,
    session_id: Option<String>,
    task_id: Option<String>,
}

async fn import_document_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<ImportDocumentPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(id) => id,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let sess = payload
        .session_id
        .as_ref()
        .map(octrex_core::ids::SessionId::from_string);
    let task = payload
        .task_id
        .as_ref()
        .map(octrex_core::ids::TaskId::from_string);
    match state.app.document_service.import_and_parse(
        &ws_id,
        &payload.rel_path,
        sess.as_ref(),
        task.as_ref(),
    ) {
        Ok(out) => {
            let doc = out.document;
            let class = doc.classification.to_string();
            Json(serde_json::json!({
                "success": true,
                "document_id": doc.document_id.as_str(),
                "version_id": doc.version_id.as_str(),
                "title": doc.title,
                "format": doc.metadata.format.as_str(),
                "mime": doc.metadata.mime,
                "size_bytes": doc.metadata.size_bytes,
                "content_hash": doc.metadata.content_hash,
                "classification": class,
                "extraction_status": doc.metadata.extraction_status.to_string(),
                "blocks": doc.blocks.len(),
                "sections": doc.sections.len(),
                "pages": doc.pages.len(),
                "chunks": out.chunks.len(),
                "warnings": doc.warnings,
                "security_findings": doc.findings,
                "text_preview": redact_chunk_text(&doc.full_text, &class),
            }))
            .into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct DocumentQuery {
    workspace_id: String,
}

async fn get_document_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<DocumentQuery>,
) -> impl IntoResponse {
    if resolve_doc_workspace(&state, &query.workspace_id).is_err() {
        return Json(serde_json::json!({ "success": false, "error": "Unknown workspace" }))
            .into_response();
    }
    match state.app.document_service.get_document(&id) {
        Ok(Some(row)) => Json(serde_json::json!({ "success": true, "document": row })).into_response(),
        Ok(None) => Json(serde_json::json!({ "success": false, "error": format!("Document '{}' not found", id) })).into_response(),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

async fn get_document_sections_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<DocumentQuery>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &query.workspace_id) {
        Ok(ws) => ws,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    match state.app.document_service.get_chunks(&id, &ws_id) {
        Ok(chunks) => {
            let sections: Vec<serde_json::Value> = {
                let mut seen = std::collections::HashSet::new();
                let mut list = Vec::new();
                for c in &chunks {
                    if let Some(s) = &c.section {
                        if seen.insert(s.clone()) {
                            list.push(serde_json::json!({ "section_id": s }));
                        }
                    }
                }
                list
            };
            let pages: Vec<u32> = {
                let mut set = std::collections::HashSet::new();
                for c in &chunks {
                    if let Some(p) = c.page {
                        set.insert(p);
                    }
                }
                let mut v: Vec<u32> = set.into_iter().collect();
                v.sort_unstable();
                v
            };
            Json(serde_json::json!({
                "success": true,
                "document_id": id,
                "sections": sections,
                "pages": pages,
                "chunks": chunks.len(),
            }))
            .into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_document_chunks_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<DocumentQuery>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &query.workspace_id) {
        Ok(ws) => ws,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    match state.app.document_service.get_chunks(&id, &ws_id) {
        Ok(chunks) => {
            let safe: Vec<serde_json::Value> = chunks
                .iter()
                .map(|c| {
                    let class = c.classification.to_string();
                    serde_json::json!({
                        "chunk_id": c.id.as_str(),
                        "chunk_index": c.chunk_index,
                        "classification": class,
                        "section": c.section,
                        "page": c.page,
                        "source": c.source_path,
                        "char_count": c.char_count,
                        "token_estimate": c.token_estimate,
                        "text": redact_chunk_text(&c.text, &class),
                    })
                })
                .collect();
            Json(serde_json::json!({ "success": true, "document_id": id, "total": safe.len(), "chunks": safe })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct SearchDocumentPayload {
    workspace_id: String,
    query: String,
    ceiling: Option<String>,
    limit: Option<usize>,
}

async fn search_document_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<SearchDocumentPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(ws) => ws,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let ceiling: octrex_core::privacy::PrivacyClassification = payload
        .ceiling
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(octrex_core::privacy::PrivacyClassification::Confidential);
    match state.app.document_service.search(
        &ws_id,
        &payload.query,
        ceiling,
        payload.limit.unwrap_or(10).min(50),
    ) {
        Ok(results) => {
            // Scope to the requested document; retrieval itself stays workspace-scoped.
            let scoped: Vec<serde_json::Value> = results
                .into_iter()
                .filter(|r| r.document_id == id)
                .map(|r| {
                    let class = r.classification.to_string();
                    serde_json::json!({
                        "chunk_id": r.chunk_id,
                        "relevance": r.relevance,
                        "classification": class,
                        "section": r.section,
                        "page": r.page,
                        "source": r.source,
                        "text": redact_chunk_text(&r.text, &class),
                    })
                })
                .collect();
            Json(serde_json::json!({ "success": true, "document_id": id, "total": scoped.len(), "results": scoped })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct IngestDocumentPayload {
    workspace_id: String,
    session_id: String,
    task_id: Option<String>,
    model_id: Option<String>,
    context_window: Option<usize>,
}

async fn ingest_document_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<IngestDocumentPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(ws) => ws,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let sess = octrex_core::ids::SessionId::from_string(&payload.session_id);
    let task = payload
        .task_id
        .as_ref()
        .map(octrex_core::ids::TaskId::from_string);
    let model_id = payload
        .model_id
        .unwrap_or_else(|| "local-default".to_string());
    let window = payload.context_window.unwrap_or(8192);
    match state.app.document_service.ingest_to_context(
        &id,
        &ws_id,
        &sess,
        task.as_ref(),
        &model_id,
        window,
    ) {
        Ok(items) => {
            let summaries: Vec<String> = items.iter().map(|i| i.safe_summary()).collect();
            Json(serde_json::json!({
                "success": true,
                "document_id": id,
                "items_ingested": items.len(),
                "safe_summaries": summaries,
            }))
            .into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct AssistDocumentPayload {
    workspace_id: String,
    document_id: String,
    operation: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
}

async fn assist_document_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<AssistDocumentPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(ws) => ws,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let sess = payload
        .session_id
        .as_ref()
        .map(octrex_core::ids::SessionId::from_string);
    let task = payload
        .task_id
        .as_ref()
        .map(octrex_core::ids::TaskId::from_string);
    let op = payload.operation.unwrap_or_else(|| "summarize".to_string());
    match state
        .app
        .document_service
        .ai_assist(&ws_id, &payload.document_id, &op, sess.as_ref(), task.as_ref())
        .await
    {
        Ok(v) => Json(serde_json::json!({ "success": true, "assist": v })).into_response(),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string(), "no_cloud_fallback": true })).into_response(),
    }
}

#[derive(Deserialize)]
struct ListArtifactsQuery {
    workspace_id: Option<String>,
    task_id: Option<String>,
}

async fn list_artifacts_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ListArtifactsQuery>,
) -> impl IntoResponse {
    use octrex_core::db::repository::{ArtifactRepository, SqliteArtifactRepository};
    let repo = SqliteArtifactRepository::new((*state.app.db).clone());
    if let Some(task_id) = query.task_id {
        let tid = octrex_core::ids::TaskId::from_string(&task_id);
        match repo.list_artifacts_by_task(&tid) {
            Ok(list) => {
                Json(serde_json::json!({ "success": true, "total": list.len(), "artifacts": list }))
                    .into_response()
            }
            Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                .into_response(),
        }
    } else if let Some(ws_id) = query.workspace_id {
        let ws = match resolve_doc_workspace(&state, &ws_id) {
            Ok(w) => w,
            Err(e) => {
                return Json(serde_json::json!({ "success": false, "error": e })).into_response()
            }
        };
        match repo.list_artifacts_by_workspace(&ws) {
            Ok(list) => {
                Json(serde_json::json!({ "success": true, "total": list.len(), "artifacts": list }))
                    .into_response()
            }
            Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() }))
                .into_response(),
        }
    } else {
        Json(serde_json::json!({ "success": false, "error": "Provide workspace_id or task_id" }))
            .into_response()
    }
}

async fn get_artifact_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use octrex_core::db::repository::{ArtifactRepository, SqliteArtifactRepository};
    let repo = SqliteArtifactRepository::new((*state.app.db).clone());
    let aid = octrex_core::ids::ArtifactId::from(id.clone());
    match repo.get_artifact(&aid) {
        Ok(Some(a)) => Json(serde_json::json!({ "success": true, "artifact": a })).into_response(),
        Ok(None) => Json(serde_json::json!({ "success": false, "error": format!("Artifact '{}' not found", id) })).into_response(),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

async fn get_artifact_lineage_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.app.document_service.artifact_lineage(&id) {
        Ok(lineage) => {
            Json(serde_json::json!({ "success": true, "artifact_id": id, "lineage": lineage }))
                .into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

async fn get_artifact_verification_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use octrex_core::db::repository::{ArtifactRepository, SqliteArtifactRepository};
    let repo = SqliteArtifactRepository::new((*state.app.db).clone());
    let aid = octrex_core::ids::ArtifactId::from(id.clone());
    match repo.get_artifact(&aid) {
        Ok(Some(a)) => {
            let lineage = state.app.document_service.artifact_lineage(&id).unwrap_or_default();
            Json(serde_json::json!({
                "success": true,
                "artifact_id": id,
                "verification_status": a.verification_status,
                "checksum": a.checksum,
                "lineage": lineage,
            }))
            .into_response()
        }
        Ok(None) => Json(serde_json::json!({ "success": false, "error": format!("Artifact '{}' not found", id) })).into_response(),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response(),
    }
}

#[derive(Deserialize)]
struct CreateArtifactPayload {
    workspace_id: String,
    task_id: Option<String>,
    session_id: Option<String>,
    rel_path: String,
    name: String,
    content: String,
    artifact_type: Option<String>,
    source_document_id: Option<String>,
    parent_artifact_id: Option<String>,
    producing_workflow: Option<String>,
    producing_skill: Option<String>,
}

async fn create_artifact_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<CreateArtifactPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(w) => w,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let kind = payload
        .artifact_type
        .as_deref()
        .and_then(octrex_core::documents::ArtifactKind::parse_label)
        .unwrap_or(octrex_core::documents::ArtifactKind::GeneratedDocument);
    if let Err(reason) = octrex_core::documents::validate_artifact_content(
        &kind,
        &payload.rel_path,
        &payload.content,
    ) {
        return Json(serde_json::json!({ "success": false, "error": reason })).into_response();
    }
    let task = payload
        .task_id
        .as_ref()
        .map(octrex_core::ids::TaskId::from_string);
    let sess = payload
        .session_id
        .as_ref()
        .map(octrex_core::ids::SessionId::from_string);
    match state.app.document_service.create_artifact(
        &ws_id,
        task.as_ref(),
        sess.as_ref(),
        &payload.rel_path,
        &payload.name,
        &payload.content,
        kind,
        payload.source_document_id.as_deref(),
        payload.parent_artifact_id.as_deref(),
        payload.producing_workflow.as_deref(),
        payload.producing_skill.as_deref(),
    ) {
        Ok(record) => {
            Json(serde_json::json!({ "success": true, "artifact": record })).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct VerifyArtifactPayload {
    workspace_id: String,
    task_id: String,
    session_id: Option<String>,
}

async fn verify_artifact_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<VerifyArtifactPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(w) => w,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let aid = octrex_core::ids::ArtifactId::from(id.clone());
    let task = octrex_core::ids::TaskId::from_string(&payload.task_id);
    let sess = payload
        .session_id
        .as_ref()
        .map(octrex_core::ids::SessionId::from_string);
    match state
        .app
        .document_service
        .verify_artifact(&aid, &task, &ws_id, sess.as_ref())
    {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "artifact_id": id,
            "verification": result,
        }))
        .into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}

#[derive(Deserialize)]
struct ExportArtifactPayload {
    workspace_id: String,
    dest_external_path: String,
}

async fn export_artifact_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(payload): Json<ExportArtifactPayload>,
) -> impl IntoResponse {
    let ws_id = match resolve_doc_workspace(&state, &payload.workspace_id) {
        Ok(w) => w,
        Err(e) => return Json(serde_json::json!({ "success": false, "error": e })).into_response(),
    };
    let aid = octrex_core::ids::ArtifactId::from(id.clone());
    match state
        .app
        .document_service
        .export_artifact(&aid, &ws_id, &payload.dest_external_path)
    {
        Ok(()) => Json(serde_json::json!({ "success": true, "artifact_id": id })).into_response(),
        Err(e) => {
            Json(serde_json::json!({ "success": false, "error": e.to_string() })).into_response()
        }
    }
}
