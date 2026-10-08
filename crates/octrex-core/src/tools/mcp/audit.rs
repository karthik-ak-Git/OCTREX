use crate::db::manager::DatabaseManager;
use crate::db::models::AuditRecord;
use crate::db::repository::AuditRepository;
use std::time::SystemTime;

pub struct McpAuditLogger;

impl McpAuditLogger {
    pub fn log_mcp_event(
        db: &DatabaseManager,
        server_id: &str,
        event_type: &str,
        success: bool,
        reason: Option<String>,
    ) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = AuditRecord {
            id: format!("audit-mcp-{}", uuid::Uuid::new_v4().simple()),
            timestamp: now,
            event_type: event_type.to_string(),
            task_id: None,
            session_id: None,
            workspace_id: None,
            actor: "mcp_boundary".to_string(),
            provider: None,
            model: None,
            route: None,
            privacy_classification: None,
            policy_source: Some("mcp_security_policy".to_string()),
            permission: None,
            tool: Some(format!("mcp:{}", server_id)),
            success,
            reason,
        };

        let repo = crate::db::repository::SqliteAuditRepository::new(db.clone());
        let _ = repo.record_audit(&record);
    }
}
