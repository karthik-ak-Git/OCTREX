use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::hardware::{CompatibilityResult, CompatibilityStatus};
use crate::models::{ModelAvailability, ModelCapability};

// ============================================================================
// LOCAL RUNTIME DESCRIPTORS
// ============================================================================

/// Provider-neutral local runtime kinds supported by Octrex.
///
/// No runtime-specific logic may leak into `ModelRouter`: the router only ever
/// sees normalized `ModelDescriptor`s plus health/lifecycle signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeType {
    Ollama,
    LlamaCppServer,
    LocalOpenAiCompatible,
    Other,
}

impl fmt::Display for LocalRuntimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalRuntimeType::Ollama => write!(f, "ollama"),
            LocalRuntimeType::LlamaCppServer => write!(f, "llama_cpp_server"),
            LocalRuntimeType::LocalOpenAiCompatible => write!(f, "local_openai_compatible"),
            LocalRuntimeType::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for LocalRuntimeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "ollama" => Ok(LocalRuntimeType::Ollama),
            "llama_cpp_server" | "llama.cpp" | "llamacpp" | "llama-cpp" => {
                Ok(LocalRuntimeType::LlamaCppServer)
            }
            "local_openai_compatible" | "openai_compatible" | "openai-compatible" => {
                Ok(LocalRuntimeType::LocalOpenAiCompatible)
            }
            "other" => Ok(LocalRuntimeType::Other),
            other => Err(format!("Unknown local runtime type: {}", other)),
        }
    }
}

/// Health of a local runtime. `Unknown` must never be treated as healthy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeHealth {
    Healthy,
    Degraded,
    Unavailable,
    Unknown,
}

impl LocalRuntimeHealth {
    pub fn is_routable(&self) -> bool {
        matches!(
            self,
            LocalRuntimeHealth::Healthy | LocalRuntimeHealth::Degraded
        )
    }
}

impl fmt::Display for LocalRuntimeHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalRuntimeHealth::Healthy => write!(f, "healthy"),
            LocalRuntimeHealth::Degraded => write!(f, "degraded"),
            LocalRuntimeHealth::Unavailable => write!(f, "unavailable"),
            LocalRuntimeHealth::Unknown => write!(f, "unknown"),
        }
    }
}

/// How the runtime executes models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalExecutionMode {
    Process,
    Endpoint,
    Managed,
}

impl fmt::Display for LocalExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalExecutionMode::Process => write!(f, "process"),
            LocalExecutionMode::Endpoint => write!(f, "endpoint"),
            LocalExecutionMode::Managed => write!(f, "managed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRuntimeDescriptor {
    pub id: String,
    pub name: String,
    pub runtime_type: LocalRuntimeType,
    /// Canonical endpoint URL, e.g. `http://127.0.0.1:11434`.
    pub endpoint: String,
    pub version: Option<String>,
    pub capabilities: Vec<ModelCapability>,
    pub execution_mode: LocalExecutionMode,
    pub health: LocalRuntimeHealth,
    pub last_checked_timestamp: Option<u64>,
    pub last_error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl LocalRuntimeDescriptor {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        runtime_type: LocalRuntimeType,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            runtime_type,
            endpoint: endpoint.into(),
            version: None,
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::Streaming,
            ],
            execution_mode: LocalExecutionMode::Endpoint,
            health: LocalRuntimeHealth::Unknown,
            last_checked_timestamp: None,
            last_error: None,
            metadata: HashMap::new(),
        }
    }
}

// ============================================================================
// LOCAL MODEL LIFECYCLE
// ============================================================================

/// Explicit lifecycle states for locally managed models.
///
/// A model is only `Running` when the backing runtime confirms it; mere
/// registration never implies availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelState {
    Discovered,
    Registered,
    Available,
    Loading,
    Loaded,
    Running,
    Unloading,
    Unavailable,
    Failed,
    Disabled,
}

impl fmt::Display for LocalModelState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalModelState::Discovered => write!(f, "discovered"),
            LocalModelState::Registered => write!(f, "registered"),
            LocalModelState::Available => write!(f, "available"),
            LocalModelState::Running => write!(f, "running"),
            LocalModelState::Loading => write!(f, "loading"),
            LocalModelState::Loaded => write!(f, "loaded"),
            LocalModelState::Unloading => write!(f, "unloading"),
            LocalModelState::Unavailable => write!(f, "unavailable"),
            LocalModelState::Failed => write!(f, "failed"),
            LocalModelState::Disabled => write!(f, "disabled"),
        }
    }
}

impl std::str::FromStr for LocalModelState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "discovered" => Ok(LocalModelState::Discovered),
            "registered" => Ok(LocalModelState::Registered),
            "available" => Ok(LocalModelState::Available),
            "loading" => Ok(LocalModelState::Loading),
            "loaded" => Ok(LocalModelState::Loaded),
            "running" => Ok(LocalModelState::Running),
            "unloading" => Ok(LocalModelState::Unloading),
            "unavailable" => Ok(LocalModelState::Unavailable),
            "failed" => Ok(LocalModelState::Failed),
            "disabled" => Ok(LocalModelState::Disabled),
            other => Err(format!("Unknown local model state: {}", other)),
        }
    }
}

impl LocalModelState {
    /// Whether the model may be selected for inference right now.
    pub fn is_routable(&self) -> bool {
        matches!(
            self,
            LocalModelState::Available | LocalModelState::Loaded | LocalModelState::Running
        )
    }

    pub fn to_availability(&self) -> ModelAvailability {
        match self {
            LocalModelState::Available | LocalModelState::Loaded | LocalModelState::Running => {
                ModelAvailability::Available
            }
            LocalModelState::Disabled => ModelAvailability::Disabled,
            LocalModelState::Discovered
            | LocalModelState::Registered
            | LocalModelState::Loading
            | LocalModelState::Unloading => ModelAvailability::Unknown,
            LocalModelState::Unavailable | LocalModelState::Failed => {
                ModelAvailability::Unavailable
            }
        }
    }
}

/// Local model record managed by `LocalModelStore`.
///
/// The canonical inference descriptor always lives in `ModelRegistry`; this
/// record carries lifecycle, provenance, and resource metadata that the
/// generic `ModelDescriptor` cannot represent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelRecord {
    pub id: String,
    pub runtime_id: String,
    pub model_identifier: String,
    pub display_name: String,
    pub registry_model_id: String,
    pub state: LocalModelState,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub capabilities: Vec<ModelCapability>,
    pub tokenizer: String,
    pub quantization: Option<String>,
    pub parameter_count_billions: Option<f64>,
    /// RAM/VRAM requirements in MB. `None` means UNKNOWN and must never be
    /// treated as compatible.
    pub required_ram_mb: Option<u64>,
    pub required_vram_mb: Option<u64>,
    pub requirements_estimated: bool,
    pub architecture: Option<String>,
    pub model_format: Option<String>,
    pub local_path: Option<String>,
    pub checksum_sha256: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
    pub health: LocalRuntimeHealth,
    pub last_error: Option<String>,
    pub created_at_timestamp: u64,
    pub updated_at_timestamp: u64,
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// COMPATIBILITY / ESTIMATION
// ============================================================================

/// Honest resource estimate. Estimates are always flagged and never presented
/// as exact measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub model_id: String,
    pub estimated_ram_mb: Option<u64>,
    pub estimated_vram_mb: Option<u64>,
    pub estimated_kv_cache_mb: Option<u64>,
    pub runtime_overhead_mb: u64,
    pub is_estimate: bool,
    pub confidence: String,
    pub assumptions: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCompatibilityReport {
    pub model_id: String,
    pub runtime_id: String,
    pub hardware: CompatibilityResult,
    pub context_ok: bool,
    pub context_detail: String,
    pub resource_estimate: ResourceEstimate,
    pub routable: bool,
    pub reasons: Vec<String>,
}

impl LocalCompatibilityReport {
    pub fn compatible_status(&self) -> CompatibilityStatus {
        self.hardware.status
    }
}

// ============================================================================
// HEALTH / METRICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRuntimeHealthReport {
    pub runtime_id: String,
    pub health: LocalRuntimeHealth,
    pub reachable: bool,
    pub models_available: usize,
    pub models_loaded: usize,
    pub latency_ms: Option<u128>,
    pub last_success_timestamp: Option<u64>,
    pub last_error: Option<String>,
    pub checked_at_timestamp: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalInferenceMetrics {
    pub call_id: String,
    pub model_id: String,
    pub runtime_id: String,
    pub time_to_first_token_ms: Option<u128>,
    pub total_latency_ms: Option<u128>,
    pub tokens_per_second: Option<f64>,
    pub prompt_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub model_load_time_ms: Option<u128>,
    pub success: bool,
    pub error: Option<String>,
}

/// Normalized local streaming events. These map 1:1 onto the existing
/// `ModelStreamEvent` vocabulary; no second SSE architecture is introduced.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum LocalStreamEvent {
    Started {
        call_id: String,
        model_id: String,
    },
    Token {
        call_id: String,
        delta: String,
    },
    Completed {
        call_id: String,
        content: String,
        metrics: Box<LocalInferenceMetrics>,
    },
    Failed {
        call_id: String,
        error: String,
    },
    Cancelled {
        call_id: String,
    },
}

/// Redacted audit event payload for local runtime activity. Never carries
/// prompts, secrets, keys, or document contents — IDs and metrics only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuditEvent {
    pub event: String,
    pub runtime_id: Option<String>,
    pub model_id: Option<String>,
    pub call_id: Option<String>,
    pub success: bool,
    pub reason: Option<String>,
    pub timestamp: u64,
}

pub fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
