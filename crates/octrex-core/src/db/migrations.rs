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
