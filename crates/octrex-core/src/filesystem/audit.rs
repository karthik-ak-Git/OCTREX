use crate::db::DatabaseManager;
use crate::error::OctrexError;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::filesystem::types::FilesystemDecision;
use tracing::info;

pub struct FilesystemAuditLogger;

impl FilesystemAuditLogger {
    /// Log filesystem security decision to SQLite database and publish event to EventBus.
    /// INVARIANT 17: NEVER store file contents, secrets, private keys, or full payloads in audit log.
    pub fn log_decision(db: &DatabaseManager, event_bus: &EventBus, decision: &FilesystemDecision) {
        let timestamp = decision.timestamp;
        let event_type_str = format!(
            "FILESYSTEM_{}",
            decision.operation.to_string().to_uppercase()
        );
        let success = decision.decision.is_allowed() as i32;

        let workspace_id_str = decision.workspace_id.as_ref().map(|id| id.as_str());

        // 1. Record in SQLite audit_records table
        let _ = db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_records (
                    id, timestamp, event_type, workspace_id, actor, route,
                    privacy_classification, policy_source, permission, tool, success, reason
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    decision.decision_id,
                    timestamp,
                    event_type_str,
                    workspace_id_str,
                    "agent_filesystem_security",
                    format!("/api/filesystem/{}", decision.operation),
                    format!("{:?}", decision.classification),
                    format!("{:?}", decision.policy_source),
                    format!("{:?}", decision.decision),
                    format!("filesystem.{}", decision.operation),
                    success,
                    decision.reason,
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Audit record insert failed: {}", e),
            })?;
            Ok::<(), OctrexError>(())
        });

        // 2. Publish event to EventBus
        let event_type = match decision.decision {
            crate::filesystem::types::FilesystemDisposition::Allow => match decision.operation {
                crate::filesystem::types::FilesystemOperation::Read => EventType::FileRead,
                crate::filesystem::types::FilesystemOperation::Create => EventType::FileCreated,
                crate::filesystem::types::FilesystemOperation::Write => EventType::FileChanged,
                crate::filesystem::types::FilesystemOperation::Delete => EventType::FileDeleted,
                _ => EventType::FileChanged,
            },
            crate::filesystem::types::FilesystemDisposition::Block => EventType::PermissionDenied,
            crate::filesystem::types::FilesystemDisposition::RequireConfirmation => {
                EventType::PermissionRequired
            }
            crate::filesystem::types::FilesystemDisposition::Unknown => EventType::PermissionDenied,
        };

        let mut envelope = EventEnvelope::new(
            event_type,
            serde_json::json!({
                "decision_id": decision.decision_id,
                "operation": decision.operation.to_string(),
                "requested_path": decision.requested_path,
                "disposition": decision.decision,
                "reason": decision.reason,
                "matched_policy": decision.matched_policy,
                "risk_level": decision.risk_level,
                "classification": decision.classification,
            }),
        );

        if let Some(ws_id) = &decision.workspace_id {
            envelope = envelope.with_workspace_id(ws_id.clone());
        }

        let _ = event_bus.publish(envelope);

        info!(
            decision_id = %decision.decision_id,
            operation = %decision.operation,
            path = %decision.requested_path,
            disposition = ?decision.decision,
            "Filesystem security decision evaluated"
        );
    }
}
