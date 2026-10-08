use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::db::{DatabaseManager, DbConfig};
use crate::events::EventBus;
use crate::filesystem::FilesystemSecurityService;
use crate::hardware::{fixture_b_4gb_nvidia, fixture_c_cpu_only, HardwareService};
use crate::local_runtime::adapter::{DiscoveredModel, MockLocalAdapter};
use crate::local_runtime::compatibility::{estimate_resources, evaluate_local_compatibility};
use crate::local_runtime::detector::DiscoveryRequest;
use crate::local_runtime::errors::LocalRuntimeError;
use crate::local_runtime::registry::ExecutionSlots;
use crate::local_runtime::security::{
    sanitize_model_metadata, validate_download_request, validate_local_endpoint,
    validate_model_file_path, validate_model_format,
};
use crate::local_runtime::service::{LocalRuntimeConfig, LocalRuntimeService, RegisterModelInput};
use crate::local_runtime::types::{
    LocalModelState, LocalRuntimeDescriptor, LocalRuntimeHealth, LocalRuntimeType,
};
use crate::models::{ModelMessage, ModelRequest};
use crate::network::NetworkSecurityService;
use crate::privacy::{PrivacyClassification, PrivacyGate};

fn test_service() -> Arc<LocalRuntimeService> {
    test_service_with_config(LocalRuntimeConfig::default())
}

fn test_service_with_config(config: LocalRuntimeConfig) -> Arc<LocalRuntimeService> {
    let db = Arc::new(DatabaseManager::new(DbConfig::in_memory()));
    db.initialize().expect("in-memory db must initialize");
    let event_bus = Arc::new(EventBus::new(256));
    let model_registry = Arc::new(crate::models::ModelRegistry::new());
    let provider_registry = Arc::new(crate::providers::ProviderRegistry::new());
    let hardware_service = Arc::new(HardwareService::real());
    let network_security = Arc::new(NetworkSecurityService::new());
    let privacy_gate = Arc::new(PrivacyGate::new());
    let filesystem_security = Arc::new(FilesystemSecurityService::new(
        db.clone(),
        event_bus.clone(),
        privacy_gate.clone(),
        network_security.clone(),
    ));
    Arc::new(LocalRuntimeService::new(
        config,
        model_registry,
        provider_registry,
        hardware_service,
        network_security,
        privacy_gate,
        filesystem_security,
        event_bus,
        db,
    ))
}

fn mock_descriptor(id: &str, healthy: bool) -> (LocalRuntimeDescriptor, MockLocalAdapter) {
    let mut desc = LocalRuntimeDescriptor::new(
        id,
        format!("Mock {}", id),
        LocalRuntimeType::Ollama,
        "http://127.0.0.1:11434",
    );
    desc.health = if healthy {
        LocalRuntimeHealth::Healthy
    } else {
        LocalRuntimeHealth::Unavailable
    };
    let mut adapter = MockLocalAdapter::healthy(id);
    adapter.healthy = healthy;
    (desc, adapter)
}

fn model_request(registry_model_id: &str, prompt: &str) -> ModelRequest {
    ModelRequest {
        model_id: registry_model_id.to_string(),
        messages: vec![ModelMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            tool_calls: None,
        }],
        system_instructions: None,
        tools: vec![],
        temperature: None,
        max_output_tokens: None,
        response_format: crate::models::ResponseFormat::Text,
        metadata: HashMap::new(),
        correlation: crate::models::CallCorrelation::default(),
    }
}

fn test_descriptor_for(
    provider_id: &str,
    model_id: &str,
    context_window: Option<u32>,
    ram: Option<u64>,
    vram: Option<u64>,
) -> crate::models::ModelDescriptor {
    crate::models::ModelDescriptor {
        id: model_id.to_string(),
        provider_id: provider_id.to_string(),
        model_identifier: model_id.to_string(),
        display_name: model_id.to_string(),
        execution_mode: crate::providers::ExecutionMode::Local,
        capabilities: vec![crate::models::ModelCapability::TextGeneration],
        context_window,
        max_output_tokens: Some(1024),
        tokenizer: crate::models::TokenizerInfo::Estimated { factor: 1.0 },
        hardware_requirements: match (ram, vram) {
            (None, None) => None,
            _ => Some(crate::models::HardwareRequirement {
                minimum_ram: ram,
                recommended_ram: None,
                minimum_vram: vram,
                recommended_vram: None,
            }),
        },
        availability: crate::models::ModelAvailability::Available,
        metadata: HashMap::new(),
    }
}

// ============================================================================
// RUNTIME DISCOVERY / ENDPOINT VALIDATION
// ============================================================================

#[test]
fn rejects_private_and_public_endpoints() {
    assert!(validate_local_endpoint("http://192.168.1.10:11434").is_err());
    assert!(validate_local_endpoint("http://10.0.0.5:8080").is_err());
    assert!(validate_local_endpoint("https://api.example.com/v1").is_err());
    assert!(validate_local_endpoint("http://127.0.0.1:11434/?api_key=secret").is_err());
}

#[test]
fn accepts_loopback_endpoints() {
    assert!(validate_local_endpoint("http://127.0.0.1:11434").is_ok());
    assert!(validate_local_endpoint("http://localhost:11434").is_ok());
    assert!(validate_local_endpoint("http://[::1]:8080").is_ok());
}

#[tokio::test]
async fn probe_rejects_non_loopback_without_network() {
    let network = NetworkSecurityService::new();
    let req = DiscoveryRequest::new("http://192.168.1.20:11434", LocalRuntimeType::Ollama, None);
    let res = crate::local_runtime::detector::probe_endpoint(&req, &network, None).await;
    assert!(res.is_err(), "private network runtime must be rejected");
}

#[tokio::test]
async fn register_runtime_rejects_invalid_endpoint() {
    let svc = test_service();
    let res = svc
        .register_runtime(LocalRuntimeType::Ollama, "http://10.1.2.3:11434", None)
        .await;
    assert!(res.is_err());
}

#[test]
fn executable_discovery_is_disabled() {
    assert!(crate::local_runtime::detector::executable_discovery().is_err());
}

// ============================================================================
// MODEL REGISTRATION / LIFECYCLE
// ============================================================================

#[tokio::test]
async fn register_and_discover_models() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-1", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-1").await.expect("discovery works");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].state, LocalModelState::Available);
    // Second discovery upserts rather than duplicating.
    let again = svc
        .discover_models("rt-1")
        .await
        .expect("rediscovery works");
    assert_eq!(again.len(), 1);
    assert_eq!(svc.list_models().len(), 1);
}

#[tokio::test]
async fn duplicate_registration_is_rejected() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-dup", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let input = RegisterModelInput {
        runtime_id: "rt-dup".to_string(),
        model_identifier: "my-model".to_string(),
        display_name: None,
        context_window: Some(8192),
        max_output_tokens: None,
        quantization: Some("Q4_K_M".to_string()),
        parameter_count_billions: Some(7.0),
        architecture: None,
        model_format: Some("gguf".to_string()),
        local_path: None,
        checksum_sha256: None,
        license: None,
        source: None,
        metadata: HashMap::new(),
        capabilities: vec![],
    };
    svc.register_model(input.clone())
        .await
        .expect("first registration");
    let err = svc
        .register_model(input)
        .await
        .expect_err("duplicate rejected");
    assert!(matches!(err, LocalRuntimeError::DuplicateModel { .. }));
}

#[tokio::test]
async fn invalid_format_and_bad_checksum_rejected() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-fmt", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let mut input = RegisterModelInput {
        runtime_id: "rt-fmt".to_string(),
        model_identifier: "m".to_string(),
        display_name: None,
        context_window: None,
        max_output_tokens: None,
        quantization: None,
        parameter_count_billions: None,
        architecture: None,
        model_format: Some("exe".to_string()),
        local_path: None,
        checksum_sha256: None,
        license: None,
        source: None,
        metadata: HashMap::new(),
        capabilities: vec![],
    };
    assert!(svc.register_model(input.clone()).await.is_err());
    input.model_format = Some("gguf".to_string());
    input.model_identifier = "m2".to_string();
    input.checksum_sha256 = Some("not-a-checksum".to_string());
    assert!(svc.register_model(input).await.is_err());
}

#[test]
fn lifecycle_transitions_are_explicit() {
    let store = crate::local_runtime::registry::LocalModelStore::new();
    let rec = local_model_record_for_tests();
    store.insert(rec.clone()).expect("insert");
    // Discovered -> Running directly is forbidden.
    let err = store
        .transition(&rec.id, LocalModelState::Running, None)
        .expect_err("must deny skip");
    assert!(matches!(err, LocalRuntimeError::LifecycleDenied { .. }));
    // Legal chain works.
    for next in [
        LocalModelState::Registered,
        LocalModelState::Available,
        LocalModelState::Loading,
        LocalModelState::Loaded,
        LocalModelState::Running,
    ] {
        store.transition(&rec.id, next, None).expect("legal step");
    }
    assert_eq!(store.get(&rec.id).unwrap().state, LocalModelState::Running);
}

fn local_model_record_for_tests() -> crate::local_runtime::types::LocalModelRecord {
    crate::local_runtime::types::LocalModelRecord {
        id: "lm-test".to_string(),
        runtime_id: "rt-test".to_string(),
        model_identifier: "test-model".to_string(),
        display_name: "Test".to_string(),
        registry_model_id: "local-runtime-rt-test:test-model".to_string(),
        state: LocalModelState::Discovered,
        context_window: Some(8192),
        max_output_tokens: Some(1024),
        capabilities: vec![],
        tokenizer: "estimated".to_string(),
        quantization: None,
        parameter_count_billions: None,
        required_ram_mb: None,
        required_vram_mb: None,
        requirements_estimated: true,
        architecture: None,
        model_format: None,
        local_path: None,
        checksum_sha256: None,
        license: None,
        source: None,
        health: LocalRuntimeHealth::Unknown,
        last_error: None,
        created_at_timestamp: 1,
        updated_at_timestamp: 1,
        metadata: HashMap::new(),
    }
}

#[test]
fn model_metadata_cannot_alter_policy() {
    let mut meta = HashMap::new();
    meta.insert("privacy_mode".to_string(), "online".to_string());
    meta.insert("routing_policy".to_string(), "always-cloud".to_string());
    meta.insert("allow_cloud".to_string(), "true".to_string());
    meta.insert("binary".to_string(), "evil.exe".to_string());
    meta.insert("display".to_string(), "hello".to_string());
    let (clean, stripped) = sanitize_model_metadata(&meta);
    assert_eq!(stripped.len(), 4);
    assert_eq!(clean.len(), 1);
    assert!(clean.contains_key("display"));
}

#[test]
fn unsafe_paths_and_formats_rejected() {
    assert!(validate_model_file_path("../evil.gguf").is_err());
    assert!(validate_model_file_path("model.exe").is_err());
    assert!(validate_model_file_path("model.gguf").is_ok());
    assert!(validate_model_format(Some("gguf")).is_ok());
    assert!(validate_model_format(Some("safetensors")).is_ok());
    assert!(validate_model_format(Some("exe")).is_err());
    assert!(validate_model_format(None).is_err());
}

// ============================================================================
// COMPATIBILITY / ESTIMATION / FIXTURES
// ============================================================================

#[test]
fn unknown_requirements_stay_unknown() {
    let est = estimate_resources("m", None, None, None, None, None);
    assert!(est.is_estimate);
    assert!(est.estimated_ram_mb.is_none());
    assert!(est.estimated_vram_mb.is_none());
    assert_eq!(est.confidence, "unknown");
}

#[test]
fn estimates_are_flagged_and_conservative() {
    let est = estimate_resources("m", Some(7.0), Some("Q4_K_M"), Some(8192), None, None);
    assert!(est.is_estimate);
    assert!(est.estimated_ram_mb.unwrap() > 0);
    assert!(est.estimated_vram_mb.unwrap() > 0);
    assert!(!est.assumptions.is_empty());
}

#[test]
fn four_gb_gpu_rejects_large_model() {
    let profile = fixture_b_4gb_nvidia();
    let mut desc = test_descriptor_for("p", "p:big", Some(8192), Some(16 * 1024), Some(12 * 1024));
    // GPU-pinned workload: 12 GB required vs 4 GB present is a hard reject.
    desc.metadata
        .insert("gpu_required".to_string(), "true".to_string());
    let report = evaluate_local_compatibility(
        &profile,
        &desc,
        "rt",
        100,
        Some(512),
        Some(70.0),
        Some("Q4_K_M"),
    );
    assert_eq!(
        report.hardware.status,
        crate::hardware::CompatibilityStatus::Incompatible
    );
    assert!(!report.routable);
}

#[test]
fn vram_shortfall_without_gpu_pin_warns_honestly() {
    // Without `gpu_required`, the engine honestly reports partial-offload
    // warnings instead of a hard reject — CPU offload is real on llama.cpp.
    let profile = fixture_b_4gb_nvidia();
    let desc = test_descriptor_for(
        "p",
        "p:big-cpu-offload",
        Some(8192),
        Some(16 * 1024),
        Some(12 * 1024),
    );
    let report = evaluate_local_compatibility(
        &profile,
        &desc,
        "rt",
        100,
        Some(512),
        Some(70.0),
        Some("Q4_K_M"),
    );
    assert_eq!(
        report.hardware.status,
        crate::hardware::CompatibilityStatus::CompatibleWithWarnings
    );
    assert!(!report.reasons.is_empty());
}

#[test]
fn eight_gb_gpu_accepts_small_model_with_warnings_or_clean() {
    let profile = crate::hardware::fixture_a_8gb_nvidia();
    let desc = test_descriptor_for("p", "p:small", Some(8192), Some(8 * 1024), Some(4 * 1024));
    let report = evaluate_local_compatibility(
        &profile,
        &desc,
        "rt",
        100,
        Some(512),
        Some(7.0),
        Some("Q4_K_M"),
    );
    assert!(report.routable);
    assert!(report.context_ok);
}

#[test]
fn cpu_only_constrains_but_honest() {
    let profile = fixture_c_cpu_only();
    let desc = test_descriptor_for("p", "p:cpu", Some(4096), Some(4 * 1024), None);
    let report =
        evaluate_local_compatibility(&profile, &desc, "rt", 100, Some(256), Some(3.0), None);
    // CPU-only without VRAM requirement: warnings expected, still routable.
    assert!(report.routable);
    assert!(!report.reasons.is_empty());
}

#[test]
fn context_overflow_is_incompatible() {
    let profile = fixture_b_4gb_nvidia();
    let desc = test_descriptor_for("p", "p:ctx", Some(2048), Some(4 * 1024), Some(2 * 1024));
    let report =
        evaluate_local_compatibility(&profile, &desc, "rt", 100_000, Some(1024), Some(3.0), None);
    assert!(!report.context_ok);
    assert!(!report.routable);
}

#[test]
fn unknown_context_window_never_routable() {
    let profile = fixture_b_4gb_nvidia();
    let desc = test_descriptor_for("p", "p:unk", None, None, None);
    let report = evaluate_local_compatibility(&profile, &desc, "rt", 100, Some(100), None, None);
    assert!(!report.context_ok);
    assert!(!report.routable);
}

#[tokio::test]
async fn dishonest_vram_claim_is_corrected() {
    let svc = test_service();
    let mut adapter = MockLocalAdapter::healthy("rt-lie");
    adapter.models = vec![DiscoveredModel {
        model_identifier: "huge-model".to_string(),
        display_name: "Huge".to_string(),
        context_window: Some(8192),
        max_output_tokens: Some(1024),
        capabilities: vec![crate::models::ModelCapability::TextGeneration],
        quantization: Some("Q4_K_M".to_string()),
        parameter_count_billions: Some(70.0),
        architecture: None,
        model_format: Some("gguf".to_string()),
        required_ram_mb: Some(512),
        required_vram_mb: Some(512),
    }];
    let (desc, adapter) = (mock_descriptor("rt-lie", true).0, adapter);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-lie").await.expect("discovery");
    let rec = &models[0];
    // Declared 512MB for a 70B model must be overridden by the estimate.
    assert!(rec.required_vram_mb.unwrap() > 512);
    assert!(rec.requirements_estimated);
    assert!(rec.metadata.contains_key("requirement_warning"));
}

// ============================================================================
// INFERENCE (mock-backed, no network)
// ============================================================================

#[tokio::test]
async fn local_execution_succeeds_and_records_metrics() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-exec", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-exec").await.expect("discovery");
    let registry_id = models[0].registry_model_id.clone();
    let resp = svc
        .execute(
            model_request(&registry_id, "hello"),
            PrivacyClassification::Public,
        )
        .await
        .expect("local execution");
    assert_eq!(resp.content, "mock local response");
    assert!(svc.recent_metrics(10).iter().any(|m| m.success));
    // Lifecycle returns to Loaded, never stuck Running.
    let rec = svc.get_model(&models[0].id).expect("record");
    assert_eq!(rec.state, LocalModelState::Loaded);
    assert!(svc.slot_occupants().is_empty());
}

#[tokio::test]
async fn streaming_delivers_deltas_and_completion() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-stream", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-stream").await.expect("discovery");
    let registry_id = models[0].registry_model_id.clone();
    let (_call_id, mut rx) = svc
        .execute_stream(
            model_request(&registry_id, "hi"),
            PrivacyClassification::Public,
        )
        .await
        .expect("stream starts");
    let mut saw_delta = false;
    let mut saw_done = false;
    let timeout = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(ev) = rx.recv().await {
            match ev {
                crate::models::ModelStreamEvent::TextDelta(_) => saw_delta = true,
                crate::models::ModelStreamEvent::Completed(_) => {
                    saw_done = true;
                    break;
                }
                crate::models::ModelStreamEvent::Failed(_) => break,
                _ => {}
            }
        }
    });
    assert!(timeout.await.is_ok());
    assert!(saw_delta && saw_done);
}

#[tokio::test]
async fn runtime_failure_marks_unavailable_no_cloud_fallback() {
    let svc = test_service();
    let (desc, mut adapter) = mock_descriptor("rt-fail", true);
    adapter.fail_invoke = true;
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-fail").await.expect("discovery");
    let registry_id = models[0].registry_model_id.clone();
    let err = svc
        .execute(
            model_request(&registry_id, "hi"),
            PrivacyClassification::Public,
        )
        .await
        .expect_err("must fail");
    let msg = err.to_string();
    assert!(msg.contains("LocalRuntimeUnavailable"), "got: {}", msg);
    assert!(!msg.to_lowercase().contains("opencode"));
    assert!(!msg.to_lowercase().contains("gemini"));
    // Metrics record the failure accurately.
    assert!(svc.recent_metrics(10).iter().any(|m| !m.success));
}

#[tokio::test]
async fn confidential_data_stays_local() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-conf", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-conf").await.expect("discovery");
    let registry_id = models[0].registry_model_id.clone();
    let resp = svc
        .execute(
            model_request(&registry_id, "secret plans"),
            PrivacyClassification::Secret,
        )
        .await
        .expect("confidential must execute locally");
    assert!(!resp.content.is_empty());
}

#[tokio::test]
async fn unknown_model_request_fails_closed() {
    let svc = test_service();
    let err = svc
        .execute(
            model_request("local-runtime-nope:ghost", "hi"),
            PrivacyClassification::Public,
        )
        .await
        .expect_err("unknown model must fail");
    assert!(err.to_string().contains("LocalRuntimeUnavailable"));
}

#[tokio::test]
async fn disabled_model_is_not_routable() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-dis", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-dis").await.expect("discovery");
    svc.disable_model(&models[0].id).expect("disable");
    let err = svc
        .execute(
            model_request(&models[0].registry_model_id, "hi"),
            PrivacyClassification::Public,
        )
        .await
        .expect_err("disabled must fail");
    assert!(err.to_string().contains("not routable"));
    // Re-enable works.
    svc.enable_model(&models[0].id).expect("enable");
}

#[test]
fn cancel_unknown_call_returns_false() {
    let svc = test_service();
    assert!(!svc.cancel("call-does-not-exist"));
}

#[tokio::test]
async fn stop_runtime_marks_models_unavailable() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-stop", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-stop").await.expect("discovery");
    svc.stop_runtime("rt-stop").await.expect("stop");
    let rec = svc.get_model(&models[0].id).expect("record");
    assert_eq!(rec.state, LocalModelState::Unavailable);
    let err = svc
        .execute(
            model_request(&models[0].registry_model_id, "hi"),
            PrivacyClassification::Public,
        )
        .await
        .expect_err("stopped runtime must fail");
    assert!(err.to_string().contains("LocalRuntimeUnavailable"));
}

#[test]
fn execution_slots_reject_second_model() {
    let slots = ExecutionSlots::new(1);
    slots.try_acquire("model-a", "call-1").expect("first slot");
    assert!(slots.try_acquire("model-b", "call-2").is_err());
    slots.release_model("model-a");
    slots.try_acquire("model-b", "call-2").expect("slot freed");
}

#[test]
fn no_automatic_cloud_fallback_guard() {
    use crate::router::{FallbackGuard, RoutingMode};
    assert!(FallbackGuard::assert_no_automatic_fallback(RoutingMode::LocalOnly, true).is_err());
    assert!(FallbackGuard::assert_no_automatic_fallback(RoutingMode::Auto, true).is_err());
    assert!(FallbackGuard::assert_no_automatic_fallback(RoutingMode::OnlineOnly, true).is_ok());
}

// ============================================================================
// DOWNLOADS
// ============================================================================

#[test]
fn download_requires_consent_and_allowlisted_source() {
    let dir = std::env::temp_dir().join("octrex-test-models");
    let _ = validate_download_request(
        "huggingface",
        "https://example.com/model.gguf",
        &dir,
        "model.gguf",
    )
    .expect_err("non-allowlisted source rejected");
    assert!(validate_download_request(
        "configured-endpoint",
        "https://example.com/model.gguf",
        &dir,
        "model.gguf",
    )
    .is_ok());
    assert!(validate_download_request(
        "configured-endpoint",
        "https://example.com/model.gguf",
        &dir,
        "../escape.gguf",
    )
    .is_err());
}

#[tokio::test]
async fn download_without_consent_is_denied() {
    let svc = test_service();
    let err = svc
        .download_model(crate::local_runtime::service::DownloadModelInput {
            source: "configured-endpoint".to_string(),
            url: "https://example.com/model.gguf".to_string(),
            file_name: "model.gguf".to_string(),
            runtime_id: None,
            model_id: None,
            expected_bytes: None,
            checksum_sha256: None,
            consent: false,
        })
        .await
        .expect_err("consent required");
    assert!(matches!(err, LocalRuntimeError::UnsafeDownload { .. }));
}

// ============================================================================
// AUDIT / EVENTS (redacted)
// ============================================================================

#[tokio::test]
async fn audit_trail_is_redacted() {
    let svc = test_service();
    let (desc, adapter) = mock_descriptor("rt-audit", true);
    svc.register_mock_runtime_for_tests(desc, adapter);
    let models = svc.discover_models("rt-audit").await.expect("discovery");
    let secret_prompt = "super-secret-prompt-xyz-123";
    let _ = svc
        .execute(
            model_request(&models[0].registry_model_id, secret_prompt),
            PrivacyClassification::Confidential,
        )
        .await;
    // No prompt content may appear in audit or event payloads: the service
    // only persists IDs + metrics. Verify via the runtime events table.
    let db_guard = svc_db(svc.clone());
    let count: i64 = db_guard
        .with_conn(|conn| {
            conn.query_row("SELECT COUNT(*) FROM model_runtime_events", [], |row| {
                row.get(0)
            })
            .map_err(|e| crate::error::OctrexError::Internal {
                message: e.to_string(),
            })
        })
        .expect("events recorded");
    assert!(count > 0);
    let leaked: i64 = db_guard
        .with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM model_runtime_events WHERE reason LIKE '%super-secret-prompt-xyz-123%'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::OctrexError::Internal {
                message: e.to_string(),
            })
        })
        .expect("query works");
    assert_eq!(leaked, 0, "prompt content must never reach audit storage");
}

fn svc_db(svc: Arc<LocalRuntimeService>) -> Arc<DatabaseManager> {
    svc.test_db()
}
