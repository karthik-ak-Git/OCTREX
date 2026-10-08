use crate::db::DatabaseManager;
use crate::error::OctrexError;
use crate::local_runtime::types::{
    now_timestamp, LocalAuditEvent, LocalModelRecord, LocalModelState, LocalRuntimeDescriptor,
    LocalRuntimeHealth, LocalRuntimeType,
};
use std::collections::HashMap;
use std::sync::Arc;

fn caps_to_json(caps: &[crate::models::ModelCapability]) -> String {
    serde_json::to_string(&caps.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>())
        .unwrap_or_else(|_| "[]".to_string())
}

fn caps_from_json(raw: &str) -> Vec<crate::models::ModelCapability> {
    serde_json::from_str::<Vec<String>>(raw)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| match s.as_str() {
            "TextGeneration" => Some(crate::models::ModelCapability::TextGeneration),
            "Vision" => Some(crate::models::ModelCapability::Vision),
            "ImageInput" => Some(crate::models::ModelCapability::ImageInput),
            "ImageOutput" => Some(crate::models::ModelCapability::ImageOutput),
            "ToolCalling" => Some(crate::models::ModelCapability::ToolCalling),
            "FunctionCalling" => Some(crate::models::ModelCapability::FunctionCalling),
            "StructuredOutput" => Some(crate::models::ModelCapability::StructuredOutput),
            "JsonOutput" => Some(crate::models::ModelCapability::JsonOutput),
            "Streaming" => Some(crate::models::ModelCapability::Streaming),
            "Embedding" => Some(crate::models::ModelCapability::Embedding),
            "CodeGeneration" => Some(crate::models::ModelCapability::CodeGeneration),
            _ => None,
        })
        .collect()
}

fn meta_to_json(meta: &HashMap<String, String>) -> String {
    serde_json::to_string(meta).unwrap_or_else(|_| "{}".to_string())
}

fn meta_from_json(raw: &str) -> HashMap<String, String> {
    serde_json::from_str(raw).unwrap_or_default()
}

/// SQLite persistence for the local runtime layer (migration v11 tables).
/// All writes are best-effort from the service layer: persistence failures
/// are surfaced but never mask the authoritative in-memory state.
pub struct SqliteLocalRuntimeRepository {
    db: Arc<DatabaseManager>,
}

impl SqliteLocalRuntimeRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> Arc<DatabaseManager> {
        self.db.clone()
    }

    pub fn save_runtime(&self, desc: &LocalRuntimeDescriptor) -> Result<(), OctrexError> {
        let caps = caps_to_json(&desc.capabilities);
        let meta = meta_to_json(&desc.metadata);
        let now = now_timestamp() as i64;
        let checked = desc.last_checked_timestamp.map(|v| v as i64);
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO local_runtimes (id, name, runtime_type, endpoint, version, capabilities_json, execution_mode, health, last_checked, last_error, metadata_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, runtime_type=excluded.runtime_type, endpoint=excluded.endpoint, version=excluded.version, capabilities_json=excluded.capabilities_json, execution_mode=excluded.execution_mode, health=excluded.health, last_checked=excluded.last_checked, last_error=excluded.last_error, metadata_json=excluded.metadata_json, updated_at=excluded.updated_at",
                rusqlite::params![
                    desc.id,
                    desc.name,
                    desc.runtime_type.to_string(),
                    desc.endpoint,
                    desc.version,
                    caps,
                    desc.execution_mode.to_string(),
                    desc.health.to_string(),
                    checked,
                    desc.last_error,
                    meta,
                    now,
                    now
                ],
            )
            .map(|_| ())
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to persist local runtime: {}", e),
            })
        })
    }

    pub fn list_runtimes(&self) -> Result<Vec<LocalRuntimeDescriptor>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, runtime_type, endpoint, version, capabilities_json, execution_mode, health, last_checked, last_error, metadata_json FROM local_runtimes")
                .map_err(|e| OctrexError::Internal {
                    message: format!("Failed to prepare local runtimes query: {}", e),
                })?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, Option<i64>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                    ))
                })
                .map_err(|e| OctrexError::Internal {
                    message: format!("Failed to query local runtimes: {}", e),
                })?;
            let mut out = Vec::new();
            for row in rows {
                let (id, name, rt, endpoint, version, caps, mode, health, checked, last_error, meta) =
                    row.map_err(|e| OctrexError::Internal {
                        message: format!("Failed to read local runtime row: {}", e),
                    })?;
                out.push(LocalRuntimeDescriptor {
                    id,
                    name,
                    runtime_type: rt.parse::<LocalRuntimeType>().unwrap_or(LocalRuntimeType::Other),
                    endpoint,
                    version,
                    capabilities: caps_from_json(&caps),
                    execution_mode: match mode.as_str() {
                        "process" => crate::local_runtime::types::LocalExecutionMode::Process,
                        "managed" => crate::local_runtime::types::LocalExecutionMode::Managed,
                        _ => crate::local_runtime::types::LocalExecutionMode::Endpoint,
                    },
                    health: match health.as_str() {
                        "healthy" => LocalRuntimeHealth::Healthy,
                        "degraded" => LocalRuntimeHealth::Degraded,
                        "unavailable" => LocalRuntimeHealth::Unavailable,
                        _ => LocalRuntimeHealth::Unknown,
                    },
                    last_checked_timestamp: checked.map(|v| v as u64),
                    last_error,
                    metadata: meta_from_json(&meta),
                });
            }
            Ok(out)
        })
    }

    pub fn save_model(&self, record: &LocalModelRecord) -> Result<(), OctrexError> {
        let caps = caps_to_json(&record.capabilities);
        let meta = meta_to_json(&record.metadata);
        let now = now_timestamp() as i64;
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO local_runtime_models (id, runtime_id, model_identifier, display_name, registry_model_id, state, context_window, max_output_tokens, capabilities_json, tokenizer, quantization, parameter_count_b, required_ram_mb, required_vram_mb, requirements_estimated, architecture, model_format, local_path, checksum_sha256, license, source, health, last_error, metadata_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)
                 ON CONFLICT(id) DO UPDATE SET runtime_id=excluded.runtime_id, model_identifier=excluded.model_identifier, display_name=excluded.display_name, registry_model_id=excluded.registry_model_id, state=excluded.state, context_window=excluded.context_window, max_output_tokens=excluded.max_output_tokens, capabilities_json=excluded.capabilities_json, tokenizer=excluded.tokenizer, quantization=excluded.quantization, parameter_count_b=excluded.parameter_count_b, required_ram_mb=excluded.required_ram_mb, required_vram_mb=excluded.required_vram_mb, requirements_estimated=excluded.requirements_estimated, architecture=excluded.architecture, model_format=excluded.model_format, local_path=excluded.local_path, checksum_sha256=excluded.checksum_sha256, license=excluded.license, source=excluded.source, health=excluded.health, last_error=excluded.last_error, metadata_json=excluded.metadata_json, updated_at=excluded.updated_at",
                rusqlite::params![
                    record.id,
                    record.runtime_id,
                    record.model_identifier,
                    record.display_name,
                    record.registry_model_id,
                    record.state.to_string(),
                    record.context_window,
                    record.max_output_tokens,
                    caps,
                    record.tokenizer,
                    record.quantization,
                    record.parameter_count_billions,
                    record.required_ram_mb.map(|v| v as i64),
                    record.required_vram_mb.map(|v| v as i64),
                    if record.requirements_estimated { 1 } else { 0 },
                    record.architecture,
                    record.model_format,
                    record.local_path,
                    record.checksum_sha256,
                    record.license,
                    record.source,
                    record.health.to_string(),
                    record.last_error,
                    meta,
                    now,
                    now
                ],
            )
            .map(|_| ())
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to persist local model: {}", e),
            })
        })
    }

    pub fn list_models(&self) -> Result<Vec<LocalModelRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, runtime_id, model_identifier, display_name, registry_model_id, state, context_window, max_output_tokens, capabilities_json, tokenizer, quantization, parameter_count_b, required_ram_mb, required_vram_mb, requirements_estimated, architecture, model_format, local_path, checksum_sha256, license, source, health, last_error, metadata_json, created_at, updated_at FROM local_runtime_models")
                .map_err(|e| OctrexError::Internal {
                    message: format!("Failed to prepare local models query: {}", e),
                })?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<u32>>(6)?,
                        row.get::<_, Option<u32>>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, Option<String>>(10)?,
                        row.get::<_, Option<f64>>(11)?,
                        row.get::<_, Option<i64>>(12)?,
                        row.get::<_, Option<i64>>(13)?,
                        row.get::<_, i64>(14)?,
                        row.get::<_, Option<String>>(15)?,
                        row.get::<_, Option<String>>(16)?,
                        row.get::<_, Option<String>>(17)?,
                        row.get::<_, Option<String>>(18)?,
                        row.get::<_, Option<String>>(19)?,
                        row.get::<_, Option<String>>(20)?,
                        row.get::<_, String>(21)?,
                        row.get::<_, Option<String>>(22)?,
                        row.get::<_, String>(23)?,
                        row.get::<_, i64>(24)?,
                        row.get::<_, i64>(25)?,
                    ))
                })
                .map_err(|e| OctrexError::Internal {
                    message: format!("Failed to query local models: {}", e),
                })?;
            let mut out = Vec::new();
            for row in rows {
                let r = row.map_err(|e| OctrexError::Internal {
                    message: format!("Failed to read local model row: {}", e),
                })?;
                out.push(LocalModelRecord {
                    id: r.0,
                    runtime_id: r.1,
                    model_identifier: r.2,
                    display_name: r.3,
                    registry_model_id: r.4,
                    state: r.5.parse::<LocalModelState>().unwrap_or(LocalModelState::Discovered),
                    context_window: r.6,
                    max_output_tokens: r.7,
                    capabilities: caps_from_json(&r.8),
                    tokenizer: r.9,
                    quantization: r.10,
                    parameter_count_billions: r.11,
                    required_ram_mb: r.12.map(|v| v as u64),
                    required_vram_mb: r.13.map(|v| v as u64),
                    requirements_estimated: r.14 != 0,
                    architecture: r.15,
                    model_format: r.16,
                    local_path: r.17,
                    checksum_sha256: r.18,
                    license: r.19,
                    source: r.20,
                    health: match r.21.as_str() {
                        "healthy" => LocalRuntimeHealth::Healthy,
                        "degraded" => LocalRuntimeHealth::Degraded,
                        "unavailable" => LocalRuntimeHealth::Unavailable,
                        _ => LocalRuntimeHealth::Unknown,
                    },
                    last_error: r.22,
                    created_at_timestamp: r.24 as u64,
                    updated_at_timestamp: r.25 as u64,
                    metadata: meta_from_json(&r.23),
                });
            }
            Ok(out)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_installation(
        &self,
        id: &str,
        model_id: Option<&str>,
        runtime_id: Option<&str>,
        source: &str,
        url: Option<&str>,
        destination: &str,
        size_bytes: Option<u64>,
        checksum: Option<&str>,
        status: &str,
        consent: bool,
    ) -> Result<(), OctrexError> {
        let now = now_timestamp() as i64;
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO model_installations (id, model_id, runtime_id, source, url, destination, size_bytes, checksum_sha256, status, consent, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET status=excluded.status, size_bytes=excluded.size_bytes, checksum_sha256=excluded.checksum_sha256, updated_at=excluded.updated_at",
                rusqlite::params![
                    id,
                    model_id,
                    runtime_id,
                    source,
                    url,
                    destination,
                    size_bytes.map(|v| v as i64),
                    checksum,
                    status,
                    if consent { 1 } else { 0 },
                    now,
                    now
                ],
            )
            .map(|_| ())
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record installation: {}", e),
            })
        })
    }

    /// Redacted local runtime event. Never stores prompts, secrets, or
    /// document contents — IDs and metrics only.
    pub fn record_runtime_event(&self, event: &LocalAuditEvent) -> Result<(), OctrexError> {
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO model_runtime_events (id, timestamp, event_type, runtime_id, model_id, call_id, success, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    format!("lre-{}", uuid::Uuid::new_v4().simple()),
                    event.timestamp as i64,
                    event.event,
                    event.runtime_id,
                    event.model_id,
                    event.call_id,
                    if event.success { 1 } else { 0 },
                    event.reason,
                ],
            )
            .map(|_| ())
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record runtime event: {}", e),
            })
        })
    }

    /// Redacted row in the shared audit trail.
    pub fn record_audit(&self, event: &LocalAuditEvent) -> Result<(), OctrexError> {
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO audit_records (id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason)
                 VALUES (?1, ?2, ?3, NULL, NULL, NULL, 'LOCAL_RUNTIME', ?4, ?5, 'local', NULL, 'local_runtime_service', NULL, NULL, ?6, ?7)",
                rusqlite::params![
                    format!("audit-{}", uuid::Uuid::new_v4().simple()),
                    event.timestamp as i64,
                    event.event,
                    event.runtime_id,
                    event.model_id,
                    if event.success { 1 } else { 0 },
                    event.reason,
                ],
            )
            .map(|_| ())
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record local audit: {}", e),
            })
        })
    }
}
