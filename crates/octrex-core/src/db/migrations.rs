use crate::error::OctrexError;
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub sql: &'static str,
}

pub static MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial Octrex schema: Workspaces, Sessions, Messages, Tasks, Task Steps, Providers, Models, Settings, Permissions, Artifacts, Events, Audit Records, Skills, MCP Connections",
        sql: r#"
CREATE TABLE IF NOT EXISTS _octrex_migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    request_id TEXT,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    started_at INTEGER,
    completed_at INTEGER,
    error_code TEXT,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS task_steps (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    objective TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    started_at INTEGER,
    completed_at INTEGER,
    error TEXT
);

CREATE TABLE IF NOT EXISTS providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model_identifier TEXT NOT NULL,
    display_name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    local_or_remote TEXT NOT NULL,
    context_window INTEGER NOT NULL DEFAULT 4096,
    capabilities TEXT,
    metadata TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    category TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (category, key)
);

CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    resource TEXT NOT NULL,
    decision TEXT NOT NULL,
    duration TEXT NOT NULL,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS artifacts (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    artifact_type TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    checksum TEXT,
    created_at INTEGER NOT NULL,
    verification_status TEXT NOT NULL DEFAULT 'UNVERIFIED'
);

CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS audit_records (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    actor TEXT NOT NULL,
    provider TEXT,
    model TEXT,
    route TEXT,
    privacy_classification TEXT,
    policy_source TEXT,
    permission TEXT,
    tool TEXT,
    success INTEGER NOT NULL,
    reason TEXT
);

CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    trust_level TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mcp_connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server_type TEXT NOT NULL,
    configuration TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_workspace ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_tasks_workspace ON tasks(workspace_id);
CREATE INDEX IF NOT EXISTS idx_tasks_session ON tasks(session_id);
CREATE INDEX IF NOT EXISTS idx_task_steps_task ON task_steps(task_id, sequence);
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider_id);
CREATE INDEX IF NOT EXISTS idx_artifacts_task ON artifacts(task_id);
CREATE INDEX IF NOT EXISTS idx_events_type_timestamp ON events(event_type, timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_records(timestamp);
CREATE INDEX IF NOT EXISTS idx_permissions_workspace ON permissions(workspace_id);
"#,
    },
    Migration {
        version: 2,
        description: "Phase 6 Privacy schema: Workspace classification, Privacy Policies, Privacy Consents, Privacy Decisions",
        sql: r#"
ALTER TABLE workspaces ADD COLUMN classification TEXT DEFAULT 'PUBLIC';

CREATE TABLE IF NOT EXISTS privacy_policies (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    priority INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    rules_json TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS privacy_consents (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    data_classification TEXT NOT NULL,
    destination_provider TEXT NOT NULL,
    destination_model TEXT NOT NULL,
    requested_mode TEXT NOT NULL,
    granted INTEGER NOT NULL DEFAULT 0,
    reason TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS privacy_decisions (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    classification TEXT NOT NULL,
    requested_mode TEXT NOT NULL,
    allowed_modes_json TEXT NOT NULL,
    decision TEXT NOT NULL,
    policy_source TEXT NOT NULL,
    policy_version INTEGER NOT NULL,
    reason TEXT NOT NULL,
    confidence TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_privacy_decisions_req ON privacy_decisions(request_id);
CREATE INDEX IF NOT EXISTS idx_privacy_consents_req ON privacy_consents(request_id);
"#,
    },
    Migration {
        version: 3,
        description: "Phase 7 Network Security schema: Network Policies, Network Rules, Network Decisions, Network Endpoints, Network Consents",
        sql: r#"
CREATE TABLE IF NOT EXISTS network_policies (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    priority INTEGER NOT NULL,
    mode TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS network_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    action TEXT NOT NULL,
    domain_pattern TEXT NOT NULL,
    protocol TEXT,
    port INTEGER,
    capability TEXT,
    provider_id TEXT,
    priority INTEGER NOT NULL DEFAULT 10,
    enabled INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS network_decisions (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    disposition TEXT NOT NULL,
    reason TEXT NOT NULL,
    matched_rule TEXT,
    policy_source TEXT NOT NULL,
    destination TEXT NOT NULL,
    protocol TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    requires_consent INTEGER NOT NULL DEFAULT 0,
    warnings_json TEXT
);

CREATE TABLE IF NOT EXISTS network_consents (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    destination TEXT NOT NULL,
    granted INTEGER NOT NULL DEFAULT 0,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_net_decisions_dest ON network_decisions(host, timestamp);
CREATE INDEX IF NOT EXISTS idx_net_rules_pattern ON network_rules(domain_pattern);
"#,
    },
    Migration {
        version: 4,
        description: "Phase 8 Filesystem Security schema: Filesystem Policies, Protected Paths, Filesystem Decisions",
        sql: r#"
CREATE TABLE IF NOT EXISTS filesystem_policies (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    read_only INTEGER NOT NULL DEFAULT 0,
    allow_delete INTEGER NOT NULL DEFAULT 1,
    allow_recursive_delete INTEGER NOT NULL DEFAULT 0,
    allow_export INTEGER NOT NULL DEFAULT 1,
    allow_import INTEGER NOT NULL DEFAULT 1,
    max_read_bytes INTEGER NOT NULL DEFAULT 10485760,
    max_write_bytes INTEGER NOT NULL DEFAULT 10485760,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS protected_paths (
    id TEXT PRIMARY KEY,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
    pattern TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS filesystem_decisions (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    operation TEXT NOT NULL,
    workspace_id TEXT,
    requested_path TEXT NOT NULL,
    resolved_path TEXT,
    decision TEXT NOT NULL,
    reason TEXT NOT NULL,
    matched_policy TEXT,
    risk_level TEXT NOT NULL,
    classification TEXT NOT NULL,
    requires_confirmation INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_fs_decisions_ws ON filesystem_decisions(workspace_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_protected_paths_ws ON protected_paths(workspace_id);
"#,
    },
    Migration {
        version: 5,
        description: "Phase 9 Tool Runtime + MCP Security schema: Capability Grants, Tool Executions",
        sql: r#"
CREATE TABLE IF NOT EXISTS capability_grants (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    capabilities_json TEXT NOT NULL,
    scope_json TEXT NOT NULL,
    expiration INTEGER,
    policy_source TEXT NOT NULL,
    consent INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_executions (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    status TEXT NOT NULL,
    decision TEXT NOT NULL,
    reason TEXT,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tool_exec_tool ON tool_executions(tool_id, created_at);
CREATE INDEX IF NOT EXISTS idx_grants_tool ON capability_grants(tool_id);
"#,
    },
    Migration {
        version: 6,
        description: "Phase 10 Context Engine schema: Context Items, Context Checkpoints, Context Compactions",
        sql: r#"
CREATE TABLE IF NOT EXISTS context_items (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    source_id TEXT,
    role TEXT NOT NULL,
    trust_level TEXT NOT NULL,
    classification TEXT NOT NULL DEFAULT 'PUBLIC',
    priority INTEGER NOT NULL DEFAULT 50,
    token_count INTEGER,
    token_count_kind TEXT NOT NULL DEFAULT 'UNKNOWN',
    content_hash TEXT,
    provenance_json TEXT,
    inclusion_reason TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    metadata_json TEXT
);

CREATE TABLE IF NOT EXISTS context_checkpoints (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    model_id TEXT NOT NULL,
    context_version INTEGER NOT NULL DEFAULT 1,
    selected_item_ids_json TEXT NOT NULL,
    summary_ids_json TEXT NOT NULL,
    task_state_version INTEGER NOT NULL DEFAULT 1,
    budget_snapshot_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS context_compactions (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    rounds INTEGER NOT NULL DEFAULT 1,
    tokens_before INTEGER NOT NULL,
    tokens_after INTEGER NOT NULL,
    items_compacted INTEGER NOT NULL DEFAULT 0,
    classification TEXT NOT NULL DEFAULT 'PUBLIC',
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ctx_items_session ON context_items(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ctx_items_task ON context_items(task_id);
CREATE INDEX IF NOT EXISTS idx_ctx_items_workspace ON context_items(workspace_id);
CREATE INDEX IF NOT EXISTS idx_ctx_checkpoints_session ON context_checkpoints(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ctx_compactions_session ON context_compactions(session_id, created_at);
"#,
    },
    Migration {
        version: 7,
        description: "Phase 12 Model Router schema: Routing Decisions, Candidate Evaluations, Evidence Records",
        sql: r#"
CREATE TABLE IF NOT EXISTS routing_decisions (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    purpose TEXT NOT NULL,
    user_privacy_mode TEXT NOT NULL,
    selected_model TEXT,
    selected_provider TEXT,
    execution_mode TEXT NOT NULL,
    decision_state TEXT NOT NULL,
    reason TEXT NOT NULL,
    candidate_count INTEGER NOT NULL DEFAULT 0,
    eligible_count INTEGER NOT NULL DEFAULT 0,
    evidence_json TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_routing_decisions_req ON routing_decisions(request_id);
CREATE INDEX IF NOT EXISTS idx_routing_decisions_task ON routing_decisions(task_id, timestamp);
"#,
    },
    Migration {
        version: 8,
        description: "Phase 11 Agent Orchestration schema: Task Plans, Orchestration Events",
        sql: r#"
CREATE TABLE IF NOT EXISTS task_plans (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    objective TEXT NOT NULL,
    constraints_json TEXT,
    steps_json TEXT NOT NULL,
    current_step INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS orchestration_events (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    step_id TEXT,
    event_type TEXT NOT NULL,
    state_from TEXT,
    state_to TEXT,
    payload_json TEXT,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_task_plans_task ON task_plans(task_id, version);
CREATE INDEX IF NOT EXISTS idx_orch_events_task ON orchestration_events(task_id, timestamp);
"#,
    },
    Migration {
        version: 9,
        description: "Phase 13 Verification & Reliability schema: Verification Runs, Verification Checks",
        sql: r#"
CREATE TABLE IF NOT EXISTS verification_runs (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    step_id TEXT,
    workspace_id TEXT,
    session_id TEXT,
    status TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0,
    repair_attempt INTEGER NOT NULL DEFAULT 0,
    warnings_json TEXT NOT NULL DEFAULT '[]',
    failures_json TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS verification_checks (
    id TEXT PRIMARY KEY,
    verification_id TEXT NOT NULL,
    name TEXT NOT NULL,
    check_type TEXT NOT NULL,
    status TEXT NOT NULL,
    severity TEXT NOT NULL,
    expected TEXT NOT NULL,
    actual TEXT NOT NULL,
    evidence_ref TEXT,
    message TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ver_runs_task ON verification_runs(task_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ver_checks_run ON verification_checks(verification_id);
"#,
    },
    Migration {
        version: 10,
        description: "Phase 14 Skills/Workflows/Memory schema: Skill Versions, Workflow Definitions/Runs, Memory Items/Candidates",
        sql: r#"
CREATE TABLE IF NOT EXISTS skill_versions (
    skill_id TEXT NOT NULL,
    version TEXT NOT NULL,
    definition_json TEXT NOT NULL,
    status TEXT NOT NULL,
    validation TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (skill_id, version)
);

CREATE TABLE IF NOT EXISTS workflow_definitions (
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    definition_json TEXT NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (id, version)
);

CREATE TABLE IF NOT EXISTS workflow_runs (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    workflow_version TEXT NOT NULL,
    task_id TEXT,
    session_id TEXT,
    workspace_id TEXT,
    status TEXT NOT NULL,
    inputs_json TEXT NOT NULL,
    outputs_json TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS memory_items (
    id TEXT PRIMARY KEY,
    mem_type TEXT NOT NULL,
    scope TEXT NOT NULL,
    workspace_id TEXT,
    project_id TEXT,
    session_id TEXT,
    task_id TEXT,
    content TEXT NOT NULL,
    classification TEXT NOT NULL,
    trust_source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0,
    provenance_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    expires_at INTEGER,
    version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS memory_candidates (
    id TEXT PRIMARY KEY,
    mem_type TEXT NOT NULL,
    scope TEXT NOT NULL,
    workspace_id TEXT,
    session_id TEXT,
    task_id TEXT,
    content TEXT NOT NULL,
    classification TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0,
    provenance_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_skill_versions_skill ON skill_versions(skill_id);
CREATE INDEX IF NOT EXISTS idx_workflow_defs_id ON workflow_definitions(id);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_wf ON workflow_runs(workflow_id, created_at);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_task ON workflow_runs(task_id);
CREATE INDEX IF NOT EXISTS idx_memory_items_ws ON memory_items(workspace_id, updated_at);
CREATE INDEX IF NOT EXISTS idx_memory_items_session ON memory_items(session_id, updated_at);
CREATE INDEX IF NOT EXISTS idx_memory_items_task ON memory_items(task_id, updated_at);
CREATE INDEX IF NOT EXISTS idx_memory_items_scope ON memory_items(scope, mem_type);
CREATE INDEX IF NOT EXISTS idx_memory_candidates_status ON memory_candidates(status, created_at);
"#,
    },
    Migration {
        version: 11,
        description: "Phase 15 Document Intelligence & Artifact Pipeline: documents, versions, sections, chunks, artifact lineage",
        sql: r#"
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    rel_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    format TEXT NOT NULL,
    mime TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    content_hash TEXT NOT NULL,
    classification TEXT NOT NULL DEFAULT 'PUBLIC',
    extraction_status TEXT NOT NULL DEFAULT 'PENDING',
    title TEXT,
    provenance_json TEXT NOT NULL DEFAULT '{}',
    warnings_json TEXT NOT NULL DEFAULT '[]',
    findings_json TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS document_versions (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL DEFAULT 1,
    content_hash TEXT NOT NULL,
    classification TEXT NOT NULL DEFAULT 'PUBLIC',
    created_at INTEGER NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS document_sections (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_id TEXT NOT NULL,
    title TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    line_start INTEGER,
    line_end INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS document_chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    text TEXT NOT NULL,
    char_count INTEGER NOT NULL DEFAULT 0,
    token_estimate INTEGER NOT NULL DEFAULT 0,
    classification TEXT NOT NULL DEFAULT 'PUBLIC',
    provenance_json TEXT NOT NULL DEFAULT '{}',
    workspace_id TEXT NOT NULL,
    session_id TEXT,
    task_id TEXT,
    section TEXT,
    page INTEGER,
    source_path TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS artifact_lineage (
    id TEXT PRIMARY KEY,
    artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    parent_artifact_id TEXT,
    source_document_id TEXT,
    source_document_version TEXT,
    producing_workflow TEXT,
    producing_skill TEXT,
    producing_model TEXT,
    producing_provider TEXT,
    classification TEXT NOT NULL DEFAULT 'PUBLIC',
    created_at INTEGER NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_documents_workspace ON documents(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(content_hash);
CREATE INDEX IF NOT EXISTS idx_documents_class ON documents(classification);
CREATE INDEX IF NOT EXISTS idx_doc_versions_doc ON document_versions(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_sections_doc ON document_sections(document_id, version_id);
CREATE INDEX IF NOT EXISTS idx_doc_chunks_doc ON document_chunks(document_id, chunk_index);
CREATE INDEX IF NOT EXISTS idx_doc_chunks_workspace ON document_chunks(workspace_id, document_id);
CREATE INDEX IF NOT EXISTS idx_doc_chunks_class ON document_chunks(classification);
CREATE INDEX IF NOT EXISTS idx_artifact_lineage_artifact ON artifact_lineage(artifact_id, created_at);
CREATE INDEX IF NOT EXISTS idx_artifact_lineage_doc ON artifact_lineage(source_document_id);
"#,
    },
    Migration {
        version: 12,
        description: "Phase 16 Local Runtime & Model Management: local runtimes, local runtime models, model installations, model runtime events",
        sql: r#"
CREATE TABLE IF NOT EXISTS local_runtimes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    runtime_type TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    version TEXT,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    execution_mode TEXT NOT NULL DEFAULT 'endpoint',
    health TEXT NOT NULL DEFAULT 'unknown',
    last_checked INTEGER,
    last_error TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS local_runtime_models (
    id TEXT PRIMARY KEY,
    runtime_id TEXT NOT NULL REFERENCES local_runtimes(id) ON DELETE CASCADE,
    model_identifier TEXT NOT NULL,
    display_name TEXT NOT NULL,
    registry_model_id TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'discovered',
    context_window INTEGER,
    max_output_tokens INTEGER,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    tokenizer TEXT NOT NULL DEFAULT 'unknown',
    quantization TEXT,
    parameter_count_b REAL,
    required_ram_mb INTEGER,
    required_vram_mb INTEGER,
    requirements_estimated INTEGER NOT NULL DEFAULT 1,
    architecture TEXT,
    model_format TEXT,
    local_path TEXT,
    checksum_sha256 TEXT,
    license TEXT,
    source TEXT,
    health TEXT NOT NULL DEFAULT 'unknown',
    last_error TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS model_installations (
    id TEXT PRIMARY KEY,
    model_id TEXT,
    runtime_id TEXT,
    source TEXT NOT NULL,
    url TEXT,
    destination TEXT NOT NULL,
    size_bytes INTEGER,
    checksum_sha256 TEXT,
    status TEXT NOT NULL,
    consent INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS model_runtime_events (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    runtime_id TEXT,
    model_id TEXT,
    call_id TEXT,
    success INTEGER NOT NULL,
    reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_local_runtimes_health ON local_runtimes(health);
CREATE INDEX IF NOT EXISTS idx_local_models_runtime ON local_runtime_models(runtime_id, state);
CREATE INDEX IF NOT EXISTS idx_local_models_registry ON local_runtime_models(registry_model_id);
CREATE INDEX IF NOT EXISTS idx_model_installations_model ON model_installations(model_id, status);
CREATE INDEX IF NOT EXISTS idx_runtime_events_model ON model_runtime_events(model_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_runtime_events_call ON model_runtime_events(call_id);
"#,
    },
];

pub fn run_migrations(conn: &mut Connection) -> Result<u32, OctrexError> {
    // Ensure migrations table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _octrex_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| OctrexError::Internal {
        message: format!("Failed to create migrations tracking table: {}", e),
    })?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _octrex_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut highest_version = current_version;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            let tx = conn.transaction().map_err(|e| OctrexError::Internal {
                message: format!(
                    "Failed to start migration transaction v{}: {}",
                    migration.version, e
                ),
            })?;

            tx.execute_batch(migration.sql)
                .map_err(|e| OctrexError::Internal {
                    message: format!("Failed to execute migration v{}: {}", migration.version, e),
                })?;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            tx.execute(
                "INSERT INTO _octrex_migrations (version, description, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![migration.version, migration.description, now],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record migration v{}: {}", migration.version, e),
            })?;

            tx.commit().map_err(|e| OctrexError::Internal {
                message: format!(
                    "Failed to commit migration transaction v{}: {}",
                    migration.version, e
                ),
            })?;

            highest_version = migration.version;
        }
    }

    Ok(highest_version)
}
