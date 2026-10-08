use crate::db::manager::DatabaseManager;
use crate::db::models::AuditRecord;
use crate::db::repository::AuditRepository;
use crate::tools::response::ToolResponse;
use crate::tools::types::ToolId;
use std::time::SystemTime;

pub struct ToolAuditLogger;

impl ToolAuditLogger {
    pub fn log_tool_execution(
        db: &DatabaseManager,
        tool_id: &ToolId,
        task_id: Option<String>,
        session_id: Option<String>,
        workspace_id: Option<String>,
        policy_source: &str,
        response: &ToolResponse,
    ) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = AuditRecord {
            id: format!("audit-tool-{}", uuid::Uuid::new_v4().simple()),
            timestamp: now,
            event_type: "TOOL_EXECUTION".to_string(),
            task_id: task_id.map(|t| crate::ids::TaskId::from(t)),
            session_id: session_id.map(|s| crate::ids::SessionId::from(s)),
            workspace_id: workspace_id.map(|w| crate::ids::WorkspaceId::from(w)),
            actor: "tool_runtime".to_string(),
            provider: None,
            model: None,
            route: None,
            privacy_classification: None,
            policy_source: Some(policy_source.to_string()),
            permission: None,
            tool: Some(tool_id.as_str().to_string()),
            success: response.error.is_none(),
            reason: response
                .error
                .clone()
                .or_else(|| Some("Execution completed".to_string())),
        };

        let repo = crate::db::repository::SqliteAuditRepository::new(db.clone());
        let _ = repo.record_audit(&record);
    }
}
