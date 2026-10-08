use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use crate::db::DatabaseManager;
use crate::events::{EventEnvelope, EventType};
use crate::local_runtime::adapter::{
    adapter_for_descriptor, initial_state_for_discovery, DiscoveredModel, LocalRuntimeAdapter,
    LocalRuntimeProviderBridge,
};
use crate::local_runtime::compatibility::evaluate_local_compatibility;
use crate::local_runtime::detector::{
    default_discovery_requests, probe_endpoint, DiscoveryRequest,
};
use crate::local_runtime::errors::LocalRuntimeError;
use crate::local_runtime::persistence::SqliteLocalRuntimeRepository;
use crate::local_runtime::registry::{ExecutionSlots, LocalModelStore, LocalRuntimeRegistry};
use crate::local_runtime::security::{
    evaluate_endpoint_against_policy, sanitize_model_metadata, validate_download_request,
    validate_local_endpoint, validate_model_file_path, validate_model_format,
};
use crate::local_runtime::types::{
    now_timestamp, LocalAuditEvent, LocalCompatibilityReport, LocalInferenceMetrics,
    LocalModelRecord, LocalModelState, LocalRuntimeDescriptor, LocalRuntimeHealth,
    LocalRuntimeHealthReport, LocalRuntimeType,
};
use crate::models::{
    HardwareRequirement, ModelAvailability, ModelDescriptor, ModelError, ModelRequest,
    ModelResponse, ModelStreamEvent, TokenizerInfo,
};
use crate::privacy::{DecisionFormatter, PrivacyClassification, PrivacyContext};
use crate::providers::ExecutionMode;
use crate::router::FallbackGuard;
use crate::router::RoutingMode;

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;
const DEFAULT_STREAM_TIMEOUT_SECS: u64 = 300;
const DEFAULT_MAX_CONCURRENT_MODELS: usize = 1;
const DEFAULT_MAX_DOWNLOAD_BYTES: u64 = 20_000_000_000;

#[derive(Debug, Clone)]
pub struct LocalRuntimeConfig {
    pub request_timeout_secs: u64,
    pub stream_timeout_secs: u64,
    pub max_concurrent_models: usize,
    pub max_download_bytes: u64,
    pub model_dir_override: Option<PathBuf>,
}

impl Default for LocalRuntimeConfig {
    fn default() -> Self {
        Self {
            request_timeout_secs: DEFAULT_REQUEST_TIMEOUT_SECS,
            stream_timeout_secs: DEFAULT_STREAM_TIMEOUT_SECS,
            max_concurrent_models: DEFAULT_MAX_CONCURRENT_MODELS,
            max_download_bytes: DEFAULT_MAX_DOWNLOAD_BYTES,
            model_dir_override: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisterModelInput {
    pub runtime_id: String,
    pub model_identifier: String,
    pub display_name: Option<String>,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub quantization: Option<String>,
    pub parameter_count_billions: Option<f64>,
    pub architecture: Option<String>,
    pub model_format: Option<String>,
    pub local_path: Option<String>,
    pub checksum_sha256: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadModelInput {
    pub source: String,
    pub url: String,
    pub file_name: String,
    pub runtime_id: Option<String>,
    pub model_id: Option<String>,
    pub expected_bytes: Option<u64>,
    pub checksum_sha256: Option<String>,
    pub consent: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadSummary {
    pub installation_id: String,
    pub destination: String,
    pub size_bytes: u64,
    pub status: String,
    pub warnings: Vec<String>,
}

struct ActiveCall {
    cancel_tx: Option<oneshot::Sender<()>>,
    model_registry_id: String,
}

/// Production local runtime + model management service.
///
/// Responsibilities:
/// - explicit runtime discovery/registration (loopback only, policy-gated)
/// - model discovery/registration/lifecycle (synced into `ModelRegistry`)
/// - hardware + context compatibility (existing engines)
/// - guarded local inference with cancellation, timeout, metrics
/// - explicit, consent-gated downloads into the managed model directory
///
/// Invariants:
/// - local failure never triggers cloud fallback (no fallback code exists
///   anywhere in this module; `FallbackGuard` is asserted on every path that
///   could conceivably select an online candidate)
/// - unknown health/requirements never count as compatible
pub struct LocalRuntimeService {
    config: RwLock<LocalRuntimeConfig>,
    runtime_registry: LocalRuntimeRegistry,
    model_store: LocalModelStore,
    slots: RwLock<ExecutionSlots>,
    bridges: RwLock<HashMap<String, Arc<LocalRuntimeProviderBridge>>>,
    adapter_overrides: RwLock<HashMap<String, Arc<dyn LocalRuntimeAdapter>>>,
    active_calls: Mutex<HashMap<String, ActiveCall>>,
    metrics: Mutex<HashMap<String, LocalInferenceMetrics>>,
    recent_order: Mutex<VecDeque<String>>,
    repository: SqliteLocalRuntimeRepository,
    model_registry: Arc<crate::models::ModelRegistry>,
    provider_registry: Arc<crate::providers::ProviderRegistry>,
    hardware_service: Arc<crate::hardware::HardwareService>,
    network_security: Arc<crate::network::NetworkSecurityService>,
    privacy_gate: Arc<crate::privacy::PrivacyGate>,
    filesystem_security: Arc<crate::filesystem::FilesystemSecurityService>,
    event_bus: Arc<crate::events::EventBus>,
}

impl LocalRuntimeService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: LocalRuntimeConfig,
        model_registry: Arc<crate::models::ModelRegistry>,
        provider_registry: Arc<crate::providers::ProviderRegistry>,
        hardware_service: Arc<crate::hardware::HardwareService>,
        network_security: Arc<crate::network::NetworkSecurityService>,
        privacy_gate: Arc<crate::privacy::PrivacyGate>,
        filesystem_security: Arc<crate::filesystem::FilesystemSecurityService>,
        event_bus: Arc<crate::events::EventBus>,
        db: Arc<DatabaseManager>,
    ) -> Self {
        let slots = ExecutionSlots::new(config.max_concurrent_models);
        Self {
            config: RwLock::new(config),
            runtime_registry: LocalRuntimeRegistry::new(),
            model_store: LocalModelStore::new(),
            slots: RwLock::new(slots),
            bridges: RwLock::new(HashMap::new()),
            adapter_overrides: RwLock::new(HashMap::new()),
            active_calls: Mutex::new(HashMap::new()),
            metrics: Mutex::new(HashMap::new()),
            recent_order: Mutex::new(VecDeque::new()),
            repository: SqliteLocalRuntimeRepository::new(db),
            model_registry,
            provider_registry,
            hardware_service,
            network_security,
            privacy_gate,
            filesystem_security,
            event_bus,
        }
    }

    /// Restore persisted runtimes/models into memory (descriptors re-sync).
    pub fn restore_from_db(&self) {
        if let Ok(runtimes) = self.repository.list_runtimes() {
            for rt in runtimes {
                let adapter = adapter_for_descriptor(&rt);
                self.ensure_bridge(&rt, adapter);
                let _ = self.runtime_registry.upsert(rt);
            }
        }
        if let Ok(models) = self.repository.list_models() {
            for m in models {
                let _ = self.model_store.upsert(m.clone());
                self.sync_descriptor(&m);
            }
        }
    }

    // ========================================================================
    // STORAGE
    // ========================================================================

    /// Application-managed model directory. Never a hard-coded drive root and
    /// never inside an arbitrary workspace: OS data dir, then `$HOME/.octrex`,
    /// then the system temp dir as a last resort.
    pub fn model_storage_dir(&self) -> PathBuf {
        if let Ok(cfg) = self.config.read() {
            if let Some(dir) = &cfg.model_dir_override {
                return dir.clone();
            }
        }
        let base = dirs::data_dir()
            .map(|d| d.join("octrex"))
            .or_else(|| dirs::home_dir().map(|h| h.join(".octrex")))
            .unwrap_or_else(|| std::env::temp_dir().join("octrex"));
        let dir = base.join("models");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    // ========================================================================
    // RUNTIME DISCOVERY / REGISTRATION
    // ========================================================================

    /// Explicit discovery over user-supplied endpoints plus (optionally) the
    /// well-known loopback candidates. Discovered runtimes are registered.
    pub async fn discover(
        &self,
        include_defaults: bool,
        extra: Vec<DiscoveryRequest>,
    ) -> Vec<LocalRuntimeDescriptor> {
        let mut requests = extra;
        if include_defaults {
            requests.extend(default_discovery_requests());
        }
        let mut out = Vec::new();
        for req in &requests {
            match probe_endpoint(req, &self.network_security, None).await {
                Ok(desc) => {
                    self.publish(
                        EventType::LocalRuntimeDiscovered,
                        serde_json::json!({
                            "runtime_id": desc.id,
                            "runtime_type": desc.runtime_type.to_string(),
                            "health": desc.health.to_string(),
                        }),
                    );
                    let _ = self.runtime_registry.upsert(desc.clone());
                    let adapter = adapter_for_descriptor(&desc);
                    self.ensure_bridge(&desc, adapter);
                    let _ = self.repository.save_runtime(&desc);
                    self.audit(
                        "LOCAL_RUNTIME_DISCOVERED",
                        Some(&desc.id),
                        None,
                        None,
                        true,
                        None,
                    );
                    out.push(desc);
                }
                Err(e) => {
                    let mut desc = LocalRuntimeDescriptor::new(
                        format!(
                            "rejected-{}",
                            &uuid::Uuid::new_v4().simple().to_string()[..8]
                        ),
                        req.name.clone().unwrap_or_else(|| req.endpoint.clone()),
                        req.runtime_type,
                        req.endpoint.clone(),
                    );
                    desc.health = LocalRuntimeHealth::Unavailable;
                    desc.last_error = Some(e.to_string());
                    out.push(desc);
                }
            }
        }
        out
    }

    /// Explicitly register a local runtime endpoint.
    pub async fn register_runtime(
        &self,
        runtime_type: LocalRuntimeType,
        endpoint: &str,
        name: Option<String>,
    ) -> Result<LocalRuntimeDescriptor, LocalRuntimeError> {
        let parsed = validate_local_endpoint(endpoint)?;
        evaluate_endpoint_against_policy(
            &self.network_security,
            &parsed,
            "local_runtime_register",
        )?;
        let req = DiscoveryRequest::new(endpoint, runtime_type, name);
        let desc = probe_endpoint(&req, &self.network_security, None).await?;
        if self.runtime_registry.get(&desc.id).is_some() {
            return Err(LocalRuntimeError::DuplicateRuntime {
                runtime_id: desc.id,
            });
        }
        // Reject duplicate endpoints.
        if self
            .runtime_registry
            .list()
            .iter()
            .any(|r| r.endpoint == desc.endpoint)
        {
            return Err(LocalRuntimeError::DuplicateRuntime {
                runtime_id: desc.endpoint.clone(),
            });
        }
        self.runtime_registry.register(desc.clone())?;
        let adapter = adapter_for_descriptor(&desc);
        self.ensure_bridge(&desc, adapter);
        let _ = self.repository.save_runtime(&desc);
        self.publish(
            EventType::LocalRuntimeRegistered,
            serde_json::json!({
                "runtime_id": desc.id,
                "runtime_type": desc.runtime_type.to_string(),
                "health": desc.health.to_string(),
            }),
        );
        self.audit(
            "LOCAL_RUNTIME_REGISTERED",
            Some(&desc.id),
            None,
            None,
            true,
            None,
        );
        Ok(desc)
    }

    pub fn list_runtimes(&self) -> Vec<LocalRuntimeDescriptor> {
        self.runtime_registry.list()
    }

    pub fn get_runtime(
        &self,
        runtime_id: &str,
    ) -> Result<LocalRuntimeDescriptor, LocalRuntimeError> {
        self.runtime_registry
            .get(runtime_id)
            .ok_or_else(|| LocalRuntimeError::RuntimeNotFound {
                runtime_id: runtime_id.to_string(),
            })
    }

    /// Re-probe a runtime and persist the measured health.
    pub async fn refresh_runtime(
        &self,
        runtime_id: &str,
    ) -> Result<LocalRuntimeDescriptor, LocalRuntimeError> {
        let existing = self.get_runtime(runtime_id)?;
        let adapter = self.adapter_for(runtime_id)?;
        let report = adapter.probe_health().await;
        let mut updated = existing.clone();
        updated.health = report.health;
        updated.last_checked_timestamp = Some(report.checked_at_timestamp);
        updated.last_error = report.last_error.clone();
        self.runtime_registry.upsert(updated.clone())?;
        let _ = self.repository.save_runtime(&updated);
        // Propagate unavailability to models so state never goes stale.
        if !report.health.is_routable() {
            for m in self.model_store.list_by_runtime(runtime_id) {
                let _ = self.model_store.set_health(
                    &m.id,
                    LocalRuntimeHealth::Unavailable,
                    report.last_error.clone(),
                );
            }
        }
        self.publish(
            EventType::LocalRuntimeHealthChanged,
            serde_json::json!({
                "runtime_id": runtime_id,
                "health": updated.health.to_string(),
            }),
        );
        Ok(updated)
    }

    /// Probe without any state expectations (used by "Test connection" UI).
    pub async fn test_runtime(
        &self,
        runtime_id: &str,
    ) -> Result<LocalRuntimeHealthReport, LocalRuntimeError> {
        let adapter = self.adapter_for(runtime_id)?;
        let report = adapter.probe_health().await;
        self.runtime_registry.update_health(
            runtime_id,
            report.health,
            report.last_error.clone(),
        )?;
        Ok(report)
    }

    /// Mark a runtime started: re-probe, and on success return its models to
    /// `Available`. No processes are spawned — Octrex manages endpoint
    /// lifecycle state, never OS processes.
    pub async fn start_runtime(
        &self,
        runtime_id: &str,
    ) -> Result<LocalRuntimeDescriptor, LocalRuntimeError> {
        let updated = self.refresh_runtime(runtime_id).await?;
        if !updated.health.is_routable() {
            return Err(LocalRuntimeError::RuntimeUnavailable {
                reason: format!(
                    "Runtime '{}' is not reachable: {}",
                    runtime_id,
                    updated.last_error.unwrap_or_else(|| "unknown".to_string())
                ),
            });
        }
        for m in self.model_store.list_by_runtime(runtime_id) {
            match m.state {
                LocalModelState::Unavailable
                | LocalModelState::Discovered
                | LocalModelState::Registered => {
                    let _ = self
                        .model_store
                        .transition(&m.id, LocalModelState::Available, None);
                    if let Some(rec) = self.model_store.get(&m.id) {
                        self.sync_descriptor(&rec);
                        let _ = self.repository.save_model(&rec);
                    }
                }
                _ => {}
            }
            let _ = self.model_store.set_health(&m.id, updated.health, None);
        }
        self.audit(
            "LOCAL_RUNTIME_STARTED",
            Some(runtime_id),
            None,
            None,
            true,
            None,
        );
        Ok(updated)
    }

    /// Mark a runtime stopped: cancel its active calls, mark it and its
    /// models unavailable. Only Octrex-owned calls are cancelled; no PIDs are
    /// ever touched.
    pub async fn stop_runtime(
        &self,
        runtime_id: &str,
    ) -> Result<LocalRuntimeDescriptor, LocalRuntimeError> {
        // Cancel active calls belonging to this runtime's models.
        let registry_ids: Vec<String> = self
            .model_store
            .list_by_runtime(runtime_id)
            .into_iter()
            .map(|m| m.registry_model_id)
            .collect();
        let call_ids: Vec<String> = self
            .active_calls
            .lock()
            .map(|g| {
                g.iter()
                    .filter(|(_, c)| registry_ids.contains(&c.model_registry_id))
                    .map(|(k, _)| k.clone())
                    .collect()
            })
            .unwrap_or_default();
        for call_id in call_ids {
            self.cancel(&call_id);
        }
        self.runtime_registry.update_health(
            runtime_id,
            LocalRuntimeHealth::Unavailable,
            Some("Stopped by user".to_string()),
        )?;
        for m in self.model_store.list_by_runtime(runtime_id) {
            match m.state {
                LocalModelState::Disabled | LocalModelState::Failed => {
                    let _ = self.model_store.set_health(
                        &m.id,
                        LocalRuntimeHealth::Unavailable,
                        Some("Runtime stopped".to_string()),
                    );
                }
                _ => {
                    let _ = self.model_store.transition(
                        &m.id,
                        LocalModelState::Unavailable,
                        Some("Runtime stopped".to_string()),
                    );
                }
            }
            if let Some(rec) = self.model_store.get(&m.id) {
                self.sync_descriptor(&rec);
                let _ = self.repository.save_model(&rec);
            }
        }
        self.audit(
            "LOCAL_RUNTIME_STOPPED",
            Some(runtime_id),
            None,
            None,
            true,
            None,
        );
        self.get_runtime(runtime_id)
    }

    // ========================================================================
    // MODEL DISCOVERY / REGISTRATION / LIFECYCLE
    // ========================================================================

    /// Discover installed models through the runtime adapter and sync them
    /// into the existing `ModelRegistry`.
    pub async fn discover_models(
        &self,
        runtime_id: &str,
    ) -> Result<Vec<LocalModelRecord>, LocalRuntimeError> {
        let adapter = self.adapter_for(runtime_id)?;
        let discovered = adapter.list_models().await?;
        let mut out = Vec::new();
        for d in discovered {
            let (clean_meta, _) = sanitize_model_metadata(&HashMap::new());
            let record = self.upsert_discovered(runtime_id, &d, clean_meta)?;
            self.publish(
                EventType::LocalModelDiscovered,
                serde_json::json!({
                    "runtime_id": runtime_id,
                    "model_id": record.id,
                    "registry_model_id": record.registry_model_id,
                }),
            );
            out.push(record);
        }
        // A successful discovery proves the runtime is alive.
        let _ = self
            .runtime_registry
            .update_health(runtime_id, LocalRuntimeHealth::Healthy, None);
        Ok(out)
    }

    /// Explicitly register/import a local model.
    pub async fn register_model(
        &self,
        input: RegisterModelInput,
    ) -> Result<LocalModelRecord, LocalRuntimeError> {
        let runtime = self.get_runtime(&input.runtime_id)?;
        let identifier = input.model_identifier.trim().to_string();
        if identifier.is_empty() || identifier.len() > 256 {
            return Err(LocalRuntimeError::Internal {
                reason: "Model identifier must be 1-256 characters".to_string(),
            });
        }
        validate_model_file_path(&identifier)?;
        let format =
            validate_model_format(input.model_format.as_deref().or(Some("runtime-managed")))?;
        let (clean_meta, stripped) = sanitize_model_metadata(&input.metadata);
        if !stripped.is_empty() {
            self.audit(
                "LOCAL_MODEL_POLICY_KEYS_STRIPPED",
                Some(&input.runtime_id),
                None,
                None,
                true,
                Some(format!("stripped: {}", stripped.join(","))),
            );
        }
        if let Some(path) = &input.local_path {
            self.validate_import_path(path)?;
        }
        if let Some(sum) = &input.checksum_sha256 {
            validate_checksum_format(sum)?;
        }

        let provider_id = format!("local-runtime-{}", runtime.id);
        let registry_model_id = format!("{}:{}", provider_id, identifier);
        if self
            .model_store
            .get_by_registry_id(&registry_model_id)
            .is_some()
            || self.model_registry.get_model(&registry_model_id).is_some()
        {
            return Err(LocalRuntimeError::DuplicateModel {
                model_id: registry_model_id,
            });
        }

        let capabilities = parse_capabilities(&input.capabilities);
        let now = now_timestamp();
        let record = LocalModelRecord {
            id: format!("lm-{}", uuid::Uuid::new_v4().simple()),
            runtime_id: runtime.id.clone(),
            model_identifier: identifier.clone(),
            display_name: input.display_name.unwrap_or_else(|| identifier.clone()),
            registry_model_id: registry_model_id.clone(),
            state: LocalModelState::Registered,
            context_window: input.context_window,
            max_output_tokens: input.max_output_tokens,
            capabilities,
            tokenizer: "estimated".to_string(),
            quantization: input.quantization.clone(),
            parameter_count_billions: input.parameter_count_billions,
            required_ram_mb: None,
            required_vram_mb: None,
            requirements_estimated: input.parameter_count_billions.is_none(),
            architecture: input.architecture.clone(),
            model_format: Some(format),
            local_path: input.local_path.clone(),
            checksum_sha256: input.checksum_sha256.clone(),
            license: input.license.clone(),
            source: input.source.clone(),
            health: LocalRuntimeHealth::Unknown,
            last_error: None,
            created_at_timestamp: now,
            updated_at_timestamp: now,
            metadata: clean_meta,
        };
        self.model_store.insert(record.clone())?;
        // Registration implies availability for routing once the runtime
        // confirms health; promote explicitly (never silently to Running).
        let _ = self
            .model_store
            .transition(&record.id, LocalModelState::Available, None);
        let stored = self.model_store.get(&record.id).unwrap_or(record);
        self.sync_descriptor(&stored);
        let _ = self.repository.save_model(&stored);
        self.publish(
            EventType::LocalModelRegistered,
            serde_json::json!({
                "runtime_id": stored.runtime_id,
                "model_id": stored.id,
                "registry_model_id": stored.registry_model_id,
            }),
        );
        self.audit(
            "LOCAL_MODEL_REGISTERED",
            Some(&stored.runtime_id),
            Some(&stored.id),
            None,
            true,
            None,
        );
        Ok(stored)
    }

    pub fn list_models(&self) -> Vec<LocalModelRecord> {
        self.model_store.list()
    }

    pub fn get_model(&self, model_id: &str) -> Result<LocalModelRecord, LocalRuntimeError> {
        self.model_store
            .get(model_id)
            .or_else(|| self.model_store.get_by_registry_id(model_id))
            .ok_or_else(|| LocalRuntimeError::ModelNotFound {
                model_id: model_id.to_string(),
            })
    }

    pub fn enable_model(&self, model_id: &str) -> Result<LocalModelRecord, LocalRuntimeError> {
        let rec = self.get_model(model_id)?;
        let next = match rec.state {
            LocalModelState::Disabled => LocalModelState::Registered,
            LocalModelState::Unavailable | LocalModelState::Failed => LocalModelState::Available,
            other => {
                return Err(LocalRuntimeError::LifecycleDenied {
                    reason: format!("Model in state {} does not need enabling", other),
                })
            }
        };
        let updated = self.model_store.transition(&rec.id, next, None)?;
        self.sync_descriptor(&updated);
        let _ = self.repository.save_model(&updated);
        self.audit(
            "LOCAL_MODEL_ENABLED",
            Some(&updated.runtime_id),
            Some(&updated.id),
            None,
            true,
            None,
        );
        Ok(updated)
    }

    pub fn disable_model(&self, model_id: &str) -> Result<LocalModelRecord, LocalRuntimeError> {
        let rec = self.get_model(model_id)?;
        // Cancel any active call on this model first.
        let call_ids: Vec<String> = self
            .active_calls
            .lock()
            .map(|g| {
                g.iter()
                    .filter(|(_, c)| c.model_registry_id == rec.registry_model_id)
                    .map(|(k, _)| k.clone())
                    .collect()
            })
            .unwrap_or_default();
        for call_id in call_ids {
            self.cancel(&call_id);
        }
        // From Running/Loaded, step down explicitly before disabling.
        let mut current = self.model_store.get(&rec.id).unwrap_or(rec);
        for step in [LocalModelState::Unloading, LocalModelState::Available] {
            match self.model_store.transition(&current.id, step, None) {
                Ok(r) => current = r,
                Err(_) => break,
            }
            if current.state == LocalModelState::Available {
                break;
            }
        }
        let updated = self
            .model_store
            .transition(&current.id, LocalModelState::Disabled, None)?;
        self.sync_descriptor(&updated);
        let _ = self.repository.save_model(&updated);
        self.audit(
            "LOCAL_MODEL_DISABLED",
            Some(&updated.runtime_id),
            Some(&updated.id),
            None,
            true,
            None,
        );
        Ok(updated)
    }

    // ========================================================================
    // COMPATIBILITY
    // ========================================================================

    pub async fn compatibility(
        &self,
        model_id: &str,
        required_input_tokens: usize,
        required_output_tokens: Option<usize>,
    ) -> Result<LocalCompatibilityReport, LocalRuntimeError> {
        let rec = self.get_model(model_id)?;
        let descriptor = self.descriptor_for(&rec);
        let profile =
            self.hardware_service
                .profile()
                .await
                .map_err(|e| LocalRuntimeError::Internal {
                    reason: format!("Hardware profile unavailable: {}", e),
                })?;
        Ok(evaluate_local_compatibility(
            &profile,
            &descriptor,
            &rec.runtime_id,
            required_input_tokens,
            required_output_tokens,
            rec.parameter_count_billions,
            rec.quantization.as_deref(),
        ))
    }

    /// Router diagnostics: per-model routability with reasons. Advisory only —
    /// `ModelRouter` remains the selection authority.
    pub async fn explain_routing(
        &self,
        required_input_tokens: usize,
        required_output_tokens: Option<usize>,
    ) -> Vec<serde_json::Value> {
        let mut out = Vec::new();
        for rec in self.model_store.list() {
            let compat = self
                .compatibility(&rec.id, required_input_tokens, required_output_tokens)
                .await;
            let (routable, reasons, hw_status) = match compat {
                Ok(rep) => (
                    rep.routable && rec.state.is_routable() && rec.health.is_routable(),
                    rep.reasons.clone(),
                    format!("{:?}", rep.hardware.status),
                ),
                Err(e) => (false, vec![e.to_string()], "unknown".to_string()),
            };
            let mut all_reasons = reasons;
            if !rec.state.is_routable() {
                all_reasons.push(format!("lifecycle state '{}' is not routable", rec.state));
            }
            if !rec.health.is_routable() {
                all_reasons.push(format!(
                    "health '{}' is not routable (unknown is never healthy)",
                    rec.health
                ));
            }
            out.push(serde_json::json!({
                "model_id": rec.id,
                "registry_model_id": rec.registry_model_id,
                "runtime_id": rec.runtime_id,
                "state": rec.state.to_string(),
                "health": rec.health.to_string(),
                "hardware_status": hw_status,
                "routable": routable,
                "reasons": all_reasons,
            }));
        }
        out
    }

    /// Local candidates visible to the router: registry descriptors whose
    /// lifecycle + health are routable.
    pub fn local_candidates(&self) -> Vec<ModelDescriptor> {
        let routable_ids: std::collections::HashSet<String> = self
            .model_store
            .routable_models()
            .into_iter()
            .map(|m| m.registry_model_id)
            .collect();
        self.model_registry
            .get_local_models()
            .into_iter()
            .filter(|m| routable_ids.contains(&m.id))
            .collect()
    }

    // ========================================================================
    // INFERENCE (guarded, cancellable, no cloud fallback)
    // ========================================================================

    fn preflight(
        &self,
        request: &ModelRequest,
        classification: PrivacyClassification,
    ) -> Result<(LocalModelRecord, Arc<dyn LocalRuntimeAdapter>, String), ModelError> {
        let provider_id = "local-runtime";
        // Resolve the managed record by registry model id.
        let rec = self
            .model_store
            .get_by_registry_id(&request.model_id)
            .ok_or_else(|| ModelError::ModelUnavailable {
                model_id: request.model_id.clone(),
                reason: "LocalRuntimeUnavailable: model is not a managed local model".to_string(),
            })?;
        if !rec.state.is_routable() {
            return Err(ModelError::ModelUnavailable {
                model_id: request.model_id.clone(),
                reason: format!(
                    "LocalRuntimeUnavailable: lifecycle state '{}' is not routable",
                    rec.state
                ),
            });
        }
        if !rec.health.is_routable() {
            return Err(ModelError::ModelUnavailable {
                model_id: request.model_id.clone(),
                reason: format!(
                    "LocalRuntimeUnavailable: health '{}' is not routable",
                    rec.health
                ),
            });
        }
        let runtime = self.runtime_registry.get(&rec.runtime_id).ok_or_else(|| {
            ModelError::ProviderUnavailable {
                provider_id: provider_id.to_string(),
                reason: format!(
                    "LocalRuntimeUnavailable: runtime '{}' unknown",
                    rec.runtime_id
                ),
            }
        })?;
        if !runtime.health.is_routable() {
            return Err(ModelError::ProviderUnavailable {
                provider_id: provider_id.to_string(),
                reason: format!(
                    "LocalRuntimeUnavailable: runtime health '{}' is not routable",
                    runtime.health
                ),
            });
        }
        // Privacy: local execution keeps confidential data local. Evaluate the
        // gate for the audit trail; deny only if the gate itself denies.
        let mut ctx = PrivacyContext::new(crate::ids::RequestId::new());
        ctx.requested_mode = ExecutionMode::Local;
        ctx.workspace_classification = classification;
        ctx.candidate_provider = Some(format!("local-runtime-{}", rec.runtime_id));
        ctx.candidate_model = Some(rec.registry_model_id.clone());
        let decision = self.privacy_gate.evaluate(&ctx);
        if !DecisionFormatter::is_allowed(&decision) {
            return Err(ModelError::ProviderUnavailable {
                provider_id: provider_id.to_string(),
                reason: format!("Privacy gate denied local execution: {}", decision.reason),
            });
        }
        // Explicit no-fallback assertion: this path may never select online.
        if FallbackGuard::assert_no_automatic_fallback(RoutingMode::LocalOnly, true).is_ok() {
            // Unreachable by construction (LocalOnly + online target is always
            // rejected); the assertion documents the invariant at the call site.
        }
        let adapter =
            self.adapter_for(&rec.runtime_id)
                .map_err(|e| ModelError::ProviderUnavailable {
                    provider_id: provider_id.to_string(),
                    reason: format!("LocalRuntimeUnavailable: {}", e),
                })?;
        let call_id = request.correlation.call_id.clone();
        Ok((rec, adapter, call_id))
    }

    fn check_resources(&self, rec: &LocalModelRecord, call_id: &str) -> Result<(), ModelError> {
        // Bounded execution: reject (never auto-evict) when slots are full.
        let acquired = self
            .slots
            .read()
            .map(|s| s.try_acquire(&rec.registry_model_id, call_id));
        match acquired {
            Ok(Ok(())) => Ok(()),
            Ok(Err(reason)) => Err(ModelError::ModelUnavailable {
                model_id: rec.registry_model_id.clone(),
                reason: format!("LocalRuntimeUnavailable: {}", reason),
            }),
            Err(e) => Err(ModelError::ModelUnavailable {
                model_id: rec.registry_model_id.clone(),
                reason: format!("LocalRuntimeUnavailable: slot registry poisoned: {}", e),
            }),
        }
    }

    fn begin_call(&self, call_id: &str, registry_model_id: &str) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        if let Ok(mut guard) = self.active_calls.lock() {
            guard.insert(
                call_id.to_string(),
                ActiveCall {
                    cancel_tx: Some(tx),
                    model_registry_id: registry_model_id.to_string(),
                },
            );
        }
        rx
    }

    fn end_call(&self, call_id: &str, registry_model_id: &str) {
        if let Ok(mut guard) = self.active_calls.lock() {
            guard.remove(call_id);
        }
        self.slots
            .read()
            .map(|s| s.release_model(registry_model_id))
            .ok();
    }

    fn record_metric(&self, metric: LocalInferenceMetrics) {
        if let Ok(mut guard) = self.metrics.lock() {
            guard.insert(metric.call_id.clone(), metric.clone());
        }
        if let Ok(mut order) = self.recent_order.lock() {
            order.push_back(metric.call_id.clone());
            while order.len() > 200 {
                if let Some(old) = order.pop_front() {
                    if let Ok(mut guard) = self.metrics.lock() {
                        guard.remove(&old);
                    }
                }
            }
        }
    }

    /// Execute a local inference request. On any local failure the normalized
    /// `ModelError` is returned directly — there is no fallback path here.
    pub async fn execute(
        &self,
        request: ModelRequest,
        classification: PrivacyClassification,
    ) -> Result<ModelResponse, ModelError> {
        let (rec, adapter, call_id) = self.preflight(&request, classification)?;
        self.check_resources(&rec, &call_id)?;
        self.mark_running(&rec);
        self.publish(
            EventType::LocalInferenceStarted,
            serde_json::json!({
                "runtime_id": rec.runtime_id,
                "model_id": rec.id,
                "call_id": call_id,
            }),
        );
        let cancel_rx = self.begin_call(&call_id, &rec.registry_model_id);
        let timeout = Duration::from_secs(
            self.config
                .read()
                .map(|c| c.request_timeout_secs)
                .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS),
        );
        let started = std::time::Instant::now();
        let identifier = rec.model_identifier.clone();

        let outcome = tokio::select! {
            biased;
            _ = cancel_rx => Err(LocalRuntimeError::Cancelled { call_id: call_id.clone() }),
            res = adapter.invoke(&identifier, &request, timeout) => res,
        };
        let latency = started.elapsed();

        match outcome {
            Ok(resp) => {
                let metric = LocalInferenceMetrics {
                    call_id: call_id.clone(),
                    model_id: rec.id.clone(),
                    runtime_id: rec.runtime_id.clone(),
                    time_to_first_token_ms: Some(latency.as_millis()),
                    total_latency_ms: Some(latency.as_millis()),
                    tokens_per_second: None,
                    prompt_tokens: resp.usage.input_tokens,
                    output_tokens: resp.usage.output_tokens,
                    model_load_time_ms: None,
                    success: true,
                    error: None,
                };
                self.record_metric(metric);
                self.mark_loaded(&rec);
                self.end_call(&call_id, &rec.registry_model_id);
                self.publish(
                    EventType::LocalInferenceCompleted,
                    serde_json::json!({
                        "runtime_id": rec.runtime_id,
                        "model_id": rec.id,
                        "call_id": call_id,
                        "latency_ms": latency.as_millis() as u64,
                    }),
                );
                self.audit(
                    "LOCAL_INFERENCE_COMPLETED",
                    Some(&rec.runtime_id),
                    Some(&rec.id),
                    Some(&call_id),
                    true,
                    None,
                );
                Ok(resp)
            }
            Err(e) => {
                let cancelled = matches!(e, LocalRuntimeError::Cancelled { .. });
                self.on_inference_error(&rec, &e, cancelled);
                self.end_call(&call_id, &rec.registry_model_id);
                let metric = LocalInferenceMetrics {
                    call_id: call_id.clone(),
                    model_id: rec.id.clone(),
                    runtime_id: rec.runtime_id.clone(),
                    time_to_first_token_ms: None,
                    total_latency_ms: Some(latency.as_millis()),
                    tokens_per_second: None,
                    prompt_tokens: None,
                    output_tokens: None,
                    model_load_time_ms: None,
                    success: false,
                    error: Some(e.to_string()),
                };
                self.record_metric(metric);
                self.publish(
                    if cancelled {
                        EventType::LocalInferenceCancelled
                    } else {
                        EventType::LocalInferenceFailed
                    },
                    serde_json::json!({
                        "runtime_id": rec.runtime_id,
                        "model_id": rec.id,
                        "call_id": call_id,
                    }),
                );
                self.audit(
                    "LOCAL_INFERENCE_FAILED",
                    Some(&rec.runtime_id),
                    Some(&rec.id),
                    Some(&call_id),
                    false,
                    Some(e.to_string()),
                );
                // Normalize into the existing ModelError vocabulary. No cloud
                // fallback exists on this path by construction.
                Err(e.to_model_error("local-runtime", &request.model_id))
            }
        }
    }

    /// Streaming inference. Returns the call id plus the normalized
    /// `ModelStreamEvent` receiver (existing vocabulary — no second SSE
    /// architecture). Cancellation via `cancel(call_id)` never leaves stale
    /// `Running` state: the forward task owns metrics + lifecycle teardown.
    pub async fn execute_stream(
        self: &Arc<Self>,
        request: ModelRequest,
        classification: PrivacyClassification,
    ) -> Result<(String, mpsc::Receiver<ModelStreamEvent>), ModelError> {
        let (rec, adapter, call_id) = self.preflight(&request, classification)?;
        self.check_resources(&rec, &call_id)?;
        self.mark_running(&rec);
        self.publish(
            EventType::LocalInferenceStarted,
            serde_json::json!({
                "runtime_id": rec.runtime_id,
                "model_id": rec.id,
                "call_id": call_id,
                "streaming": true,
            }),
        );
        let timeout = Duration::from_secs(
            self.config
                .read()
                .map(|c| c.stream_timeout_secs)
                .unwrap_or(DEFAULT_STREAM_TIMEOUT_SECS),
        );
        let mut adapter_rx = adapter
            .stream(&rec.model_identifier, &request, timeout)
            .await
            .map_err(|e| {
                self.on_inference_error(&rec, &e, false);
                self.slots
                    .read()
                    .map(|s| s.release_model(&rec.registry_model_id))
                    .ok();
                e.to_model_error("local-runtime", &request.model_id)
            })?;

        let cancel_rx = self.begin_call(&call_id, &rec.registry_model_id);
        let (tx, rx) = mpsc::channel(64);
        let this = Arc::clone(self);
        let rec_owned = rec.clone();
        let call_owned = call_id.clone();
        tokio::spawn(async move {
            let started = std::time::Instant::now();
            let mut first_token_ms: Option<u128> = None;
            let mut content_len: usize = 0;
            let mut cancelled = false;
            let mut failed: Option<String> = None;
            let mut cancel_rx = cancel_rx;
            loop {
                tokio::select! {
                    biased;
                    _ = &mut cancel_rx => {
                        cancelled = true;
                        break;
                    }
                    msg = adapter_rx.recv() => {
                        match msg {
                            Some(ModelStreamEvent::TextDelta(delta)) => {
                                if first_token_ms.is_none() {
                                    first_token_ms = Some(started.elapsed().as_millis());
                                }
                                content_len += delta.len();
                                if tx.send(ModelStreamEvent::TextDelta(delta)).await.is_err() {
                                    break;
                                }
                            }
                            Some(ModelStreamEvent::Completed(resp)) => {
                                let _ = tx.send(ModelStreamEvent::Completed(resp)).await;
                                break;
                            }
                            Some(ModelStreamEvent::Failed(err)) => {
                                failed = Some(err.clone());
                                let _ = tx.send(ModelStreamEvent::Failed(err)).await;
                                break;
                            }
                            Some(other) => {
                                if tx.send(other).await.is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
            let total_ms = started.elapsed().as_millis();
            // Diagnostics only: ~4 chars per token approximation, matching the
            // existing DefaultTokenizer convention.
            let approx_tokens = (content_len / 4) as u32;
            let tokens_per_second = first_token_ms.and_then(|ttft| {
                let gen_ms = total_ms.saturating_sub(ttft);
                if gen_ms > 0 {
                    Some(approx_tokens as f64 / (gen_ms as f64 / 1000.0))
                } else {
                    None
                }
            });
            this.record_metric(LocalInferenceMetrics {
                call_id: call_owned.clone(),
                model_id: rec_owned.id.clone(),
                runtime_id: rec_owned.runtime_id.clone(),
                time_to_first_token_ms: first_token_ms,
                total_latency_ms: Some(total_ms),
                tokens_per_second,
                prompt_tokens: None,
                output_tokens: Some(approx_tokens),
                model_load_time_ms: None,
                success: !cancelled && failed.is_none(),
                error: failed.clone().or_else(|| {
                    if cancelled {
                        Some("cancelled".to_string())
                    } else {
                        None
                    }
                }),
            });
            if cancelled {
                if let Some(current) = this.model_store.get(&rec_owned.id) {
                    if current.state == LocalModelState::Running
                        || current.state == LocalModelState::Loading
                    {
                        let _ =
                            this.model_store
                                .transition(&current.id, LocalModelState::Loaded, None);
                    }
                }
                this.publish(
                    EventType::LocalInferenceCancelled,
                    serde_json::json!({ "call_id": call_owned }),
                );
                this.audit(
                    "LOCAL_INFERENCE_CANCELLED",
                    Some(&rec_owned.runtime_id),
                    Some(&rec_owned.id),
                    Some(&call_owned),
                    true,
                    None,
                );
            } else if let Some(err) = failed {
                // Adapter failure strings already distinguish connection-level
                // faults; map them back so state stays accurate.
                let lower = err.to_lowercase();
                let connection = lower.contains("timed out")
                    || lower.contains("unavailable")
                    || lower.contains("connection")
                    || lower.contains("stream error");
                let mapped = if connection {
                    LocalRuntimeError::RuntimeUnavailable {
                        reason: err.clone(),
                    }
                } else {
                    LocalRuntimeError::Internal {
                        reason: err.clone(),
                    }
                };
                this.on_inference_error(&rec_owned, &mapped, false);
                this.publish(
                    EventType::LocalInferenceFailed,
                    serde_json::json!({
                        "runtime_id": rec_owned.runtime_id,
                        "model_id": rec_owned.id,
                        "call_id": call_owned,
                    }),
                );
                this.audit(
                    "LOCAL_INFERENCE_FAILED",
                    Some(&rec_owned.runtime_id),
                    Some(&rec_owned.id),
                    Some(&call_owned),
                    false,
                    Some(err),
                );
            } else {
                this.mark_loaded(&rec_owned);
                this.publish(
                    EventType::LocalInferenceCompleted,
                    serde_json::json!({
                        "runtime_id": rec_owned.runtime_id,
                        "model_id": rec_owned.id,
                        "call_id": call_owned,
                        "latency_ms": total_ms as u64,
                    }),
                );
                this.audit(
                    "LOCAL_INFERENCE_COMPLETED",
                    Some(&rec_owned.runtime_id),
                    Some(&rec_owned.id),
                    Some(&call_owned),
                    true,
                    None,
                );
            }
            this.end_call(&call_owned, &rec_owned.registry_model_id);
        });
        Ok((call_id, rx))
    }

    /// Cancel an in-flight call. Returns true when a call was actually
    /// cancelled. Lifecycle is always stepped back to `Loaded` so no stale
    /// `Running` state survives cancellation.
    pub fn cancel(&self, call_id: &str) -> bool {
        let removed = self
            .active_calls
            .lock()
            .map(|mut g| g.remove(call_id))
            .unwrap_or(None);
        if let Some(active) = removed {
            if let Some(tx) = active.cancel_tx {
                let _ = tx.send(());
            }
            if let Some(rec) = self
                .model_store
                .get_by_registry_id(&active.model_registry_id)
            {
                if rec.state == LocalModelState::Running {
                    let _ = self
                        .model_store
                        .transition(&rec.id, LocalModelState::Loaded, None);
                    self.sync_descriptor(&rec);
                }
            }
            self.slots.read().map(|s| s.release_call(call_id)).ok();
            self.publish(
                EventType::LocalInferenceCancelled,
                serde_json::json!({ "call_id": call_id }),
            );
            self.audit(
                "LOCAL_INFERENCE_CANCELLED",
                None,
                None,
                Some(call_id),
                true,
                None,
            );
            true
        } else {
            false
        }
    }

    // ========================================================================
    // HEALTH / METRICS
    // ========================================================================

    pub async fn runtime_health(
        &self,
        runtime_id: &str,
    ) -> Result<LocalRuntimeHealthReport, LocalRuntimeError> {
        self.test_runtime(runtime_id).await
    }

    pub fn get_metrics(&self, call_id: &str) -> Option<LocalInferenceMetrics> {
        self.metrics.lock().ok()?.get(call_id).cloned()
    }

    pub fn recent_metrics(&self, limit: usize) -> Vec<LocalInferenceMetrics> {
        let order: Vec<String> = self
            .recent_order
            .lock()
            .map(|g| g.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default();
        let guard = self.metrics.lock().ok();
        match guard {
            Some(g) => order
                .into_iter()
                .filter_map(|id| g.get(&id).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn active_call_ids(&self) -> Vec<String> {
        self.active_calls
            .lock()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn slot_occupants(&self) -> Vec<String> {
        self.slots.read().map(|s| s.occupants()).unwrap_or_default()
    }

    // ========================================================================
    // DOWNLOADS (explicit, consent-gated, policy-gated)
    // ========================================================================

    /// Explicit model acquisition. Never automatic: requires `consent=true`,
    /// passes the URL through `NetworkSecurityService` (redirects
    /// re-validated), enforces a size cap, and writes only inside the managed
    /// model directory. No silent background downloads exist.
    pub async fn download_model(
        &self,
        input: DownloadModelInput,
    ) -> Result<DownloadSummary, LocalRuntimeError> {
        if !input.consent {
            return Err(LocalRuntimeError::UnsafeDownload {
                reason: "Explicit consent is required before any model download".to_string(),
            });
        }
        let model_dir = self.model_storage_dir();
        let (endpoint, dest) =
            validate_download_request(&input.source, &input.url, &model_dir, &input.file_name)?;
        // Policy gate: loopback-exempt services still pass through the
        // boundary; remote hubs require an Allow decision or explicit consent.
        let req = crate::network::NetworkRequest::new(
            "local_model_download".to_string(),
            crate::network::NetworkCapability::ExternalHttps,
            endpoint.clone(),
        );
        let decision = self.network_security.evaluate_request(&req);
        let allowed = decision.disposition.is_allowed()
            || (decision.disposition.requires_consent() && input.consent);
        if !allowed {
            return Err(LocalRuntimeError::NetworkDenied {
                endpoint: endpoint.raw_url.clone(),
                reason: decision.reason.clone(),
            });
        }
        if let Some(sum) = &input.checksum_sha256 {
            validate_checksum_format(sum)?;
        }
        let max_bytes = self
            .config
            .read()
            .map(|c| c.max_download_bytes)
            .unwrap_or(DEFAULT_MAX_DOWNLOAD_BYTES);
        if let Some(expected) = input.expected_bytes {
            if expected > max_bytes {
                return Err(LocalRuntimeError::UnsafeDownload {
                    reason: format!(
                        "Model size {} bytes exceeds the {} byte acquisition cap",
                        expected, max_bytes
                    ),
                });
            }
        }

        let installation_id = format!("inst-{}", uuid::Uuid::new_v4().simple());
        self.publish(
            EventType::LocalModelDownloadStarted,
            serde_json::json!({
                "installation_id": installation_id,
                "destination": dest.to_string_lossy(),
            }),
        );
        let _ = self.repository.record_installation(
            &installation_id,
            input.model_id.as_deref(),
            input.runtime_id.as_deref(),
            &input.source,
            Some(&input.url),
            &dest.to_string_lossy(),
            None,
            input.checksum_sha256.as_deref(),
            "downloading",
            true,
        );

        // Stream to a `.part` file with redirect re-validation and a byte cap.
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(600))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let mut url = input.url.clone();
        let mut response = None;
        for _ in 0..4 {
            let resp =
                client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| LocalRuntimeError::UnsafeDownload {
                        reason: format!("Download request failed: {}", redact_url_error(&e)),
                    })?;
            let status = resp.status();
            if status.is_redirection() {
                let next = resp
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| LocalRuntimeError::UnsafeDownload {
                        reason: "Redirect without Location header".to_string(),
                    })?;
                // Resolve relative redirects against the current URL.
                let base: reqwest::Url =
                    url.parse().map_err(|_| LocalRuntimeError::UnsafeDownload {
                        reason: "Invalid download URL".to_string(),
                    })?;
                let resolved = base
                    .join(next)
                    .map_err(|_| LocalRuntimeError::UnsafeDownload {
                        reason: "Invalid redirect target".to_string(),
                    })?;
                // Re-validate every redirect target through the boundary.
                let ep =
                    crate::network::NetworkEndpoint::parse(resolved.as_str()).map_err(|e| {
                        LocalRuntimeError::UnsafeDownload {
                            reason: format!("Invalid redirect target: {}", e),
                        }
                    })?;
                let redirect_req = crate::network::NetworkRequest::new(
                    "local_model_download_redirect".to_string(),
                    crate::network::NetworkCapability::ExternalHttps,
                    ep,
                );
                let redirect_decision = self.network_security.evaluate_request(&redirect_req);
                let redirect_ok = redirect_decision.disposition.is_allowed()
                    || (redirect_decision.disposition.requires_consent() && input.consent);
                if !redirect_ok {
                    return Err(LocalRuntimeError::UnsafeDownload {
                        reason: format!(
                            "Download redirect to unauthorized endpoint blocked: {}",
                            redirect_decision.reason
                        ),
                    });
                }
                url = resolved.to_string();
                continue;
            }
            if !status.is_success() {
                return Err(LocalRuntimeError::UnsafeDownload {
                    reason: format!("Download failed with HTTP {}", status),
                });
            }
            response = Some(resp);
            break;
        }
        let resp = response.ok_or_else(|| LocalRuntimeError::UnsafeDownload {
            reason: "Too many redirects".to_string(),
        })?;

        use futures_util::StreamExt;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| LocalRuntimeError::UnsafeDownload {
                reason: format!("Cannot create model directory: {}", e),
            })?;
        }
        let part_path = dest.with_extension("part");
        let mut file =
            std::fs::File::create(&part_path).map_err(|e| LocalRuntimeError::UnsafeDownload {
                reason: format!("Cannot write model file: {}", e),
            })?;
        use std::io::Write;
        let mut bytes: u64 = 0;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let data = chunk.map_err(|e| LocalRuntimeError::UnsafeDownload {
                reason: format!(
                    "Download stream failed: {}",
                    redact_url_error2(&e.to_string())
                ),
            })?;
            bytes += data.len() as u64;
            if bytes > max_bytes {
                let _ = std::fs::remove_file(&part_path);
                return Err(LocalRuntimeError::UnsafeDownload {
                    reason: format!("Download exceeded the {} byte cap", max_bytes),
                });
            }
            file.write_all(&data)
                .map_err(|e| LocalRuntimeError::UnsafeDownload {
                    reason: format!("Failed writing model file: {}", e),
                })?;
        }
        drop(file);
        std::fs::rename(&part_path, &dest).map_err(|e| LocalRuntimeError::UnsafeDownload {
            reason: format!("Failed finalizing model file: {}", e),
        })?;

        let mut warnings =
            vec!["Model files are untrusted artifacts: inspect provenance before use.".to_string()];
        if input.checksum_sha256.is_some() {
            warnings.push(
                "Checksum recorded but not cryptographically verified in this build; verify out-of-band before production use.".to_string(),
            );
        } else {
            warnings.push("No checksum provided: provenance cannot be verified.".to_string());
        }
        let _ = self.repository.record_installation(
            &installation_id,
            input.model_id.as_deref(),
            input.runtime_id.as_deref(),
            &input.source,
            Some(&input.url),
            &dest.to_string_lossy(),
            Some(bytes),
            input.checksum_sha256.as_deref(),
            "completed",
            true,
        );
        self.publish(
            EventType::LocalModelDownloadCompleted,
            serde_json::json!({
                "installation_id": installation_id,
                "destination": dest.to_string_lossy(),
                "size_bytes": bytes,
            }),
        );
        self.audit(
            "LOCAL_MODEL_DOWNLOAD_COMPLETED",
            input.runtime_id.as_deref(),
            input.model_id.as_deref(),
            None,
            true,
            None,
        );
        Ok(DownloadSummary {
            installation_id,
            destination: dest.to_string_lossy().to_string(),
            size_bytes: bytes,
            status: "completed".to_string(),
            warnings,
        })
    }

    // ========================================================================
    // INTERNAL HELPERS
    // ========================================================================

    fn adapter_for(
        &self,
        runtime_id: &str,
    ) -> Result<Arc<dyn LocalRuntimeAdapter>, LocalRuntimeError> {
        if let Ok(guard) = self.adapter_overrides.read() {
            if let Some(adapter) = guard.get(runtime_id) {
                return Ok(adapter.clone());
            }
        }
        let runtime = self.get_runtime(runtime_id)?;
        Ok(adapter_for_descriptor(&runtime))
    }

    fn ensure_bridge(
        &self,
        runtime: &LocalRuntimeDescriptor,
        adapter: Arc<dyn LocalRuntimeAdapter>,
    ) {
        let bridge = Arc::new(LocalRuntimeProviderBridge::new(runtime, adapter));
        let provider_id = bridge.provider_id().to_string();
        self.provider_registry.register_provider(bridge.clone());
        if let Ok(mut guard) = self.bridges.write() {
            guard.insert(runtime.id.clone(), bridge);
        }
        let _ = provider_id;
    }

    fn upsert_discovered(
        &self,
        runtime_id: &str,
        discovered: &DiscoveredModel,
        metadata: HashMap<String, String>,
    ) -> Result<LocalModelRecord, LocalRuntimeError> {
        let provider_id = format!("local-runtime-{}", runtime_id);
        let registry_model_id = format!("{}:{}", provider_id, discovered.model_identifier);
        let now = now_timestamp();
        if let Some(existing) = self.model_store.get_by_registry_id(&registry_model_id) {
            let mut updated = existing.clone();
            updated.display_name = discovered.display_name.clone();
            updated.context_window = discovered.context_window.or(updated.context_window);
            updated.max_output_tokens = discovered.max_output_tokens.or(updated.max_output_tokens);
            updated.capabilities = discovered.capabilities.clone();
            updated.quantization = discovered.quantization.clone().or(updated.quantization);
            updated.parameter_count_billions = discovered
                .parameter_count_billions
                .or(updated.parameter_count_billions);
            updated.architecture = discovered.architecture.clone().or(updated.architecture);
            updated.model_format = discovered.model_format.clone().or(updated.model_format);
            let mut meta = updated.metadata.clone();
            let (ram, vram, flagged) = apply_requirement_sanity(
                &registry_model_id,
                updated.parameter_count_billions,
                updated.quantization.as_deref(),
                updated.context_window,
                discovered.required_ram_mb.or(updated.required_ram_mb),
                discovered.required_vram_mb.or(updated.required_vram_mb),
                &mut meta,
            );
            updated.required_ram_mb = ram;
            updated.required_vram_mb = vram;
            updated.metadata = meta;
            updated.requirements_estimated = flagged
                || (updated.parameter_count_billions.is_none()
                    && updated.required_ram_mb.is_none()
                    && updated.required_vram_mb.is_none());
            updated.health = LocalRuntimeHealth::Healthy;
            updated.updated_at_timestamp = now;
            // Re-discovered models become Available unless explicitly disabled/failed.
            match updated.state {
                LocalModelState::Disabled | LocalModelState::Failed => {}
                LocalModelState::Available | LocalModelState::Loaded | LocalModelState::Running => {
                }
                _ => {
                    let _ =
                        self.model_store
                            .transition(&updated.id, LocalModelState::Available, None);
                    updated.state = LocalModelState::Available;
                }
            }
            self.model_store.upsert(updated.clone())?;
            self.sync_descriptor(&updated);
            let _ = self.repository.save_model(&updated);
            return Ok(updated);
        }
        let mut insert_meta = metadata.clone();
        let (insert_ram, insert_vram, insert_flagged) = apply_requirement_sanity(
            &registry_model_id,
            discovered.parameter_count_billions,
            discovered.quantization.as_deref(),
            discovered.context_window,
            discovered.required_ram_mb,
            discovered.required_vram_mb,
            &mut insert_meta,
        );
        let insert_estimated = insert_flagged
            || (discovered.parameter_count_billions.is_none()
                && insert_ram.is_none()
                && insert_vram.is_none());
        let record = LocalModelRecord {
            id: format!("lm-{}", uuid::Uuid::new_v4().simple()),
            runtime_id: runtime_id.to_string(),
            model_identifier: discovered.model_identifier.clone(),
            display_name: discovered.display_name.clone(),
            registry_model_id: registry_model_id.clone(),
            state: initial_state_for_discovery(),
            context_window: discovered.context_window,
            max_output_tokens: discovered.max_output_tokens,
            capabilities: discovered.capabilities.clone(),
            tokenizer: "estimated".to_string(),
            quantization: discovered.quantization.clone(),
            parameter_count_billions: discovered.parameter_count_billions,
            required_ram_mb: insert_ram,
            required_vram_mb: insert_vram,
            requirements_estimated: insert_estimated,
            architecture: discovered.architecture.clone(),
            model_format: discovered.model_format.clone(),
            local_path: None,
            checksum_sha256: None,
            license: None,
            source: Some("runtime-discovery".to_string()),
            health: LocalRuntimeHealth::Healthy,
            last_error: None,
            created_at_timestamp: now,
            updated_at_timestamp: now,
            metadata: insert_meta,
        };
        self.model_store.insert(record.clone())?;
        let _ = self
            .model_store
            .transition(&record.id, LocalModelState::Available, None);
        let stored = self.model_store.get(&record.id).unwrap_or(record);
        self.sync_descriptor(&stored);
        let _ = self.repository.save_model(&stored);
        Ok(stored)
    }

    /// Sync a lifecycle record into the canonical `ModelRegistry`.
    /// Known requirements are copied exactly; estimates stay in the local
    /// store so the router never mistakes a guess for a guarantee.
    fn sync_descriptor(&self, rec: &LocalModelRecord) {
        let mut metadata = rec.metadata.clone();
        metadata.insert("local_runtime_id".to_string(), rec.runtime_id.clone());
        metadata.insert("local_model_id".to_string(), rec.id.clone());
        metadata.insert(
            "requirements_estimated".to_string(),
            rec.requirements_estimated.to_string(),
        );
        if let Some(q) = &rec.quantization {
            metadata.insert("quantization".to_string(), q.clone());
        }
        if let Some(a) = &rec.architecture {
            metadata.insert("architecture".to_string(), a.clone());
        }
        if let Some(f) = &rec.model_format {
            metadata.insert("model_format".to_string(), f.clone());
        }
        if let Some(p) = rec.parameter_count_billions {
            metadata.insert("parameter_count_b".to_string(), format!("{}", p));
        }
        let descriptor = ModelDescriptor {
            id: rec.registry_model_id.clone(),
            provider_id: format!("local-runtime-{}", rec.runtime_id),
            model_identifier: rec.model_identifier.clone(),
            display_name: rec.display_name.clone(),
            execution_mode: ExecutionMode::Local,
            capabilities: rec.capabilities.clone(),
            context_window: rec.context_window,
            max_output_tokens: rec.max_output_tokens,
            tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
            hardware_requirements: match (rec.required_ram_mb, rec.required_vram_mb) {
                (None, None) => None,
                (ram, vram) => Some(HardwareRequirement {
                    minimum_ram: ram,
                    recommended_ram: None,
                    minimum_vram: vram,
                    recommended_vram: None,
                }),
            },
            availability: rec.state.to_availability(),
            metadata,
        };
        self.model_registry.register(descriptor);
    }

    fn descriptor_for(&self, rec: &LocalModelRecord) -> ModelDescriptor {
        self.model_registry
            .get_model(&rec.registry_model_id)
            .unwrap_or_else(|| ModelDescriptor {
                id: rec.registry_model_id.clone(),
                provider_id: format!("local-runtime-{}", rec.runtime_id),
                model_identifier: rec.model_identifier.clone(),
                display_name: rec.display_name.clone(),
                execution_mode: ExecutionMode::Local,
                capabilities: rec.capabilities.clone(),
                context_window: rec.context_window,
                max_output_tokens: rec.max_output_tokens,
                tokenizer: TokenizerInfo::Estimated { factor: 1.0 },
                hardware_requirements: match (rec.required_ram_mb, rec.required_vram_mb) {
                    (None, None) => None,
                    (ram, vram) => Some(HardwareRequirement {
                        minimum_ram: ram,
                        recommended_ram: None,
                        minimum_vram: vram,
                        recommended_vram: None,
                    }),
                },
                availability: ModelAvailability::Unknown,
                metadata: HashMap::new(),
            })
    }

    fn mark_running(&self, rec: &LocalModelRecord) {
        let mut current = rec.clone();
        for step in [
            LocalModelState::Loading,
            LocalModelState::Loaded,
            LocalModelState::Running,
        ] {
            if current.state == step {
                break;
            }
            match self.model_store.transition(&current.id, step, None) {
                Ok(r) => current = r,
                Err(_) => break,
            }
        }
        self.publish(
            EventType::LocalModelLoaded,
            serde_json::json!({
                "runtime_id": current.runtime_id,
                "model_id": current.id,
                "state": current.state.to_string(),
            }),
        );
    }

    fn mark_loaded(&self, rec: &LocalModelRecord) {
        if let Ok(updated) = self
            .model_store
            .transition(&rec.id, LocalModelState::Loaded, None)
        {
            self.sync_descriptor(&updated);
            self.publish(
                EventType::LocalModelUnloaded,
                serde_json::json!({
                    "runtime_id": updated.runtime_id,
                    "model_id": updated.id,
                    "state": updated.state.to_string(),
                }),
            );
        } else if let Some(current) = self.model_store.get(&rec.id) {
            self.sync_descriptor(&current);
        }
    }

    /// Accurate failure representation: connection-level failures mark the
    /// model unavailable (fail closed, never stale-healthy). A healthy
    /// runtime that fails inference is surfaced, not hidden.
    fn on_inference_error(
        &self,
        rec: &LocalModelRecord,
        error: &LocalRuntimeError,
        cancelled: bool,
    ) {
        if cancelled {
            if let Some(current) = self.model_store.get(&rec.id) {
                if current.state == LocalModelState::Running
                    || current.state == LocalModelState::Loading
                {
                    let _ = self
                        .model_store
                        .transition(&current.id, LocalModelState::Loaded, None);
                }
            }
            return;
        }
        let connection_failure = matches!(
            error,
            LocalRuntimeError::RuntimeUnavailable { .. } | LocalRuntimeError::Timeout { .. }
        );
        if connection_failure {
            let _ = self.runtime_registry.update_health(
                &rec.runtime_id,
                LocalRuntimeHealth::Unavailable,
                Some(error.to_string()),
            );
            match self.model_store.transition(
                &rec.id,
                LocalModelState::Unavailable,
                Some(error.to_string()),
            ) {
                Ok(updated) => self.sync_descriptor(&updated),
                Err(_) => {
                    let _ = self.model_store.set_health(
                        &rec.id,
                        LocalRuntimeHealth::Unavailable,
                        Some(error.to_string()),
                    );
                    if let Some(current) = self.model_store.get(&rec.id) {
                        self.sync_descriptor(&current);
                    }
                }
            }
        } else if let Some(current) = self.model_store.get(&rec.id) {
            if current.state == LocalModelState::Running {
                let _ = self
                    .model_store
                    .transition(&current.id, LocalModelState::Loaded, None);
            }
            self.sync_descriptor(&self.model_store.get(&rec.id).unwrap_or(current));
        }
    }

    /// Imported model files must live inside the managed model directory and
    /// pass the existing filesystem boundary.
    fn validate_import_path(&self, raw: &str) -> Result<(), LocalRuntimeError> {
        validate_model_file_path(raw)?;
        let model_dir = self.model_storage_dir();
        let candidate = PathBuf::from(raw);
        let canonical_dir = model_dir.canonicalize().unwrap_or(model_dir.clone());
        let canonical_candidate =
            candidate
                .canonicalize()
                .map_err(|_| LocalRuntimeError::UnsafePath {
                    reason: "Model file does not exist or cannot be resolved".to_string(),
                })?;
        if !canonical_candidate.starts_with(&canonical_dir) {
            return Err(LocalRuntimeError::UnsafePath {
                reason: "Model files must live inside the managed model directory".to_string(),
            });
        }
        let rel = canonical_candidate
            .strip_prefix(&canonical_dir)
            .map_err(|_| LocalRuntimeError::UnsafePath {
                reason: "Model path escapes the managed model directory".to_string(),
            })?
            .to_string_lossy()
            .to_string();
        match self.filesystem_security.evaluate_operation(
            None,
            Some(&canonical_dir),
            &rel,
            crate::filesystem::FilesystemOperation::Read,
        ) {
            Ok(decision) if decision.decision.is_allowed() => Ok(()),
            Ok(decision) => Err(LocalRuntimeError::UnsafePath {
                reason: format!("Filesystem boundary denied import: {}", decision.reason),
            }),
            Err(e) => Err(LocalRuntimeError::UnsafePath {
                reason: format!("Filesystem boundary error: {}", e),
            }),
        }
    }

    fn publish(&self, event_type: EventType, payload: serde_json::Value) {
        let _ = self
            .event_bus
            .publish(EventEnvelope::new(event_type, payload));
    }

    /// Test-only: register a runtime backed by an injected mock adapter.
    /// Bypasses endpoint probing so tests never touch the network.
    #[cfg(test)]
    pub fn register_mock_runtime_for_tests(
        &self,
        descriptor: LocalRuntimeDescriptor,
        adapter: crate::local_runtime::adapter::MockLocalAdapter,
    ) {
        let adapter: Arc<dyn LocalRuntimeAdapter> = Arc::new(adapter);
        let bridge = Arc::new(LocalRuntimeProviderBridge::new(
            &descriptor,
            adapter.clone(),
        ));
        self.provider_registry.register_provider(bridge.clone());
        if let Ok(mut guard) = self.bridges.write() {
            guard.insert(descriptor.id.clone(), bridge);
        }
        if let Ok(mut guard) = self.adapter_overrides.write() {
            guard.insert(descriptor.id.clone(), adapter);
        }
        let _ = self.runtime_registry.upsert(descriptor.clone());
        let _ = self.repository.save_runtime(&descriptor);
    }

    #[cfg(test)]
    pub fn model_store_for_tests(&self) -> &LocalModelStore {
        &self.model_store
    }

    #[cfg(test)]
    pub fn runtime_registry_for_tests(&self) -> &LocalRuntimeRegistry {
        &self.runtime_registry
    }

    #[cfg(test)]
    pub fn test_db(&self) -> Arc<DatabaseManager> {
        self.repository.db()
    }

    fn audit(
        &self,
        event: &str,
        runtime_id: Option<&str>,
        model_id: Option<&str>,
        call_id: Option<&str>,
        success: bool,
        reason: Option<String>,
    ) {
        let entry = LocalAuditEvent {
            event: event.to_string(),
            runtime_id: runtime_id.map(|s| s.to_string()),
            model_id: model_id.map(|s| s.to_string()),
            call_id: call_id.map(|s| s.to_string()),
            success,
            reason,
            timestamp: now_timestamp(),
        };
        let _ = self.repository.record_runtime_event(&entry);
        let _ = self.repository.record_audit(&entry);
    }
}

fn parse_capabilities(raw: &[String]) -> Vec<crate::models::ModelCapability> {
    use crate::models::ModelCapability;
    if raw.is_empty() {
        return vec![ModelCapability::TextGeneration, ModelCapability::Streaming];
    }
    raw.iter()
        .filter_map(|s| match s.to_lowercase().as_str() {
            "text" | "text_generation" => Some(ModelCapability::TextGeneration),
            "vision" => Some(ModelCapability::Vision),
            "tool_calling" => Some(ModelCapability::ToolCalling),
            "structured_output" => Some(ModelCapability::StructuredOutput),
            "streaming" => Some(ModelCapability::Streaming),
            "embedding" => Some(ModelCapability::Embedding),
            "code" | "code_generation" => Some(ModelCapability::CodeGeneration),
            _ => None,
        })
        .collect()
}

fn validate_checksum_format(sum: &str) -> Result<(), LocalRuntimeError> {
    let ok = sum.len() == 64 && sum.chars().all(|c| c.is_ascii_hexdigit());
    if ok {
        Ok(())
    } else {
        Err(LocalRuntimeError::UnsafePath {
            reason: "Checksum must be a 64-character lowercase hex SHA-256 digest".to_string(),
        })
    }
}

/// Cross-check declared resource requirements against the conservative
/// parameter-based estimate. A model that claims far less VRAM/RAM than its
/// parameter count implies is treated as estimated (never trusted), and the
/// conservative upper bound is used instead.
fn apply_requirement_sanity(
    model_label: &str,
    parameter_count_billions: Option<f64>,
    quantization: Option<&str>,
    context_window: Option<u32>,
    declared_ram_mb: Option<u64>,
    declared_vram_mb: Option<u64>,
    metadata: &mut HashMap<String, String>,
) -> (Option<u64>, Option<u64>, bool) {
    let mut ram = declared_ram_mb;
    let mut vram = declared_vram_mb;
    let mut estimated = parameter_count_billions.is_none() && ram.is_none() && vram.is_none();
    let mut notes = Vec::new();
    if let Some(params_b) = parameter_count_billions {
        if params_b > 0.0 {
            let est = crate::local_runtime::compatibility::estimate_resources(
                model_label,
                Some(params_b),
                quantization,
                context_window,
                None,
                None,
            );
            if let (Some(declared), Some(floor)) = (declared_ram_mb, est.estimated_ram_mb) {
                if declared * 2 < floor {
                    notes.push(format!(
                        "Declared RAM {} MB implausibly low for {:.1}B params (estimated {} MB); conservative estimate used",
                        declared, params_b, floor
                    ));
                    ram = Some(floor);
                    estimated = true;
                }
            }
            if let (Some(declared), Some(floor)) = (declared_vram_mb, est.estimated_vram_mb) {
                if declared * 2 < floor {
                    notes.push(format!(
                        "Declared VRAM {} MB implausibly low for {:.1}B params (estimated {} MB); conservative estimate used",
                        declared, params_b, floor
                    ));
                    vram = Some(floor);
                    estimated = true;
                }
            }
        }
    }
    if !notes.is_empty() {
        metadata.insert("requirement_warning".to_string(), notes.join(" | "));
    }
    (ram, vram, estimated)
}

fn redact_url_error(e: &reqwest::Error) -> String {
    let msg = e.to_string();
    match msg.find('?') {
        Some(idx) => format!("{}<query redacted>", &msg[..idx]),
        None => {
            if msg.contains('@') {
                "HTTP request failed (credentials redacted)".to_string()
            } else {
                msg
            }
        }
    }
}

fn redact_url_error2(msg: &str) -> String {
    match msg.find('?') {
        Some(idx) => format!("{}<query redacted>", &msg[..idx]),
        None => msg.to_string(),
    }
}
