use crate::db::manager::DatabaseManager;
use crate::db::models::AuditRecord;
use crate::error::OctrexError;
use crate::ids::{SessionId, TaskId, WorkspaceId};
use rusqlite::params;

pub trait AuditRepository: Send + Sync {
    fn record_audit(&self, record: &AuditRecord) -> Result<AuditRecord, OctrexError>;
    fn get_audit(&self, id: &str) -> Result<Option<AuditRecord>, OctrexError>;
    fn list_audits(&self, limit: usize) -> Result<Vec<AuditRecord>, OctrexError>;
    fn list_audits_by_task(&self, task_id: &TaskId) -> Result<Vec<AuditRecord>, OctrexError>;
}

pub struct SqliteAuditRepository {
    db: DatabaseManager,
}

impl SqliteAuditRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl AuditRepository for SqliteAuditRepository {
    fn record_audit(&self, record: &AuditRecord) -> Result<AuditRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let task_id_str = record.task_id.as_ref().map(|t| t.as_str());
            let sess_id_str = record.session_id.as_ref().map(|s| s.as_str());
            let ws_id_str = record.workspace_id.as_ref().map(|w| w.as_str());

            conn.execute(
                "INSERT INTO audit_records (id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    record.id,
                    record.timestamp,
                    record.event_type,
                    task_id_str,
                    sess_id_str,
                    ws_id_str,
                    record.actor,
                    record.provider,
                    record.model,
                    record.route,
                    record.privacy_classification,
                    record.policy_source,
                    record.permission,
                    record.tool,
                    if record.success { 1 } else { 0 },
                    record.reason
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record audit log: {}", e),
            })?;
            Ok(record.clone())
        })
    }

    fn get_audit(&self, id: &str) -> Result<Option<AuditRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason FROM audit_records WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let timestamp: u64 = row.get(1)?;
                let event_type: String = row.get(2)?;
                let task_id_str: Option<String> = row.get(3)?;
                let sess_id_str: Option<String> = row.get(4)?;
                let ws_id_str: Option<String> = row.get(5)?;
                let actor: String = row.get(6)?;
                let provider: Option<String> = row.get(7)?;
                let model: Option<String> = row.get(8)?;
                let route: Option<String> = row.get(9)?;
                let privacy_classification: Option<String> = row.get(10)?;
                let policy_source: Option<String> = row.get(11)?;
                let permission: Option<String> = row.get(12)?;
                let tool: Option<String> = row.get(13)?;
                let success_val: i32 = row.get(14)?;
                let reason: Option<String> = row.get(15)?;

                Ok(AuditRecord {
                    id: id_str,
                    timestamp,
                    event_type,
                    task_id: task_id_str.map(TaskId::from),
                    session_id: sess_id_str.map(SessionId::from),
                    workspace_id: ws_id_str.map(WorkspaceId::from),
                    actor,
                    provider,
                    model,
                    route,
                    privacy_classification,
                    policy_source,
                    permission,
                    tool,
                    success: success_val != 0,
                    reason,
                })
            });

            match result {
                Ok(a) => Ok(Some(a)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_audits(&self, limit: usize) -> Result<Vec<AuditRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason FROM audit_records ORDER BY timestamp DESC LIMIT ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![limit as i64], |row| {
                    let id_str: String = row.get(0)?;
                    let timestamp: u64 = row.get(1)?;
                    let event_type: String = row.get(2)?;
                    let task_id_str: Option<String> = row.get(3)?;
                    let sess_id_str: Option<String> = row.get(4)?;
                    let ws_id_str: Option<String> = row.get(5)?;
                    let actor: String = row.get(6)?;
                    let provider: Option<String> = row.get(7)?;
                    let model: Option<String> = row.get(8)?;
                    let route: Option<String> = row.get(9)?;
                    let privacy_classification: Option<String> = row.get(10)?;
                    let policy_source: Option<String> = row.get(11)?;
                    let permission: Option<String> = row.get(12)?;
                    let tool: Option<String> = row.get(13)?;
                    let success_val: i32 = row.get(14)?;
                    let reason: Option<String> = row.get(15)?;

                    Ok(AuditRecord {
                        id: id_str,
                        timestamp,
                        event_type,
                        task_id: task_id_str.map(TaskId::from),
                        session_id: sess_id_str.map(SessionId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        actor,
                        provider,
                        model,
                        route,
                        privacy_classification,
                        policy_source,
                        permission,
                        tool,
                        success: success_val != 0,
                        reason,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(list)
        })
    }

    fn list_audits_by_task(&self, task_id: &TaskId) -> Result<Vec<AuditRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason FROM audit_records WHERE task_id = ?1 ORDER BY timestamp DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![task_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let timestamp: u64 = row.get(1)?;
                    let event_type: String = row.get(2)?;
                    let task_id_str: Option<String> = row.get(3)?;
                    let sess_id_str: Option<String> = row.get(4)?;
                    let ws_id_str: Option<String> = row.get(5)?;
                    let actor: String = row.get(6)?;
                    let provider: Option<String> = row.get(7)?;
                    let model: Option<String> = row.get(8)?;
                    let route: Option<String> = row.get(9)?;
                    let privacy_classification: Option<String> = row.get(10)?;
                    let policy_source: Option<String> = row.get(11)?;
                    let permission: Option<String> = row.get(12)?;
                    let tool: Option<String> = row.get(13)?;
                    let success_val: i32 = row.get(14)?;
                    let reason: Option<String> = row.get(15)?;

                    Ok(AuditRecord {
                        id: id_str,
                        timestamp,
                        event_type,
                        task_id: task_id_str.map(TaskId::from),
                        session_id: sess_id_str.map(SessionId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        actor,
                        provider,
                        model,
                        route,
                        privacy_classification,
                        policy_source,
                        permission,
                        tool,
                        success: success_val != 0,
                        reason,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(list)
        })
    }
}
