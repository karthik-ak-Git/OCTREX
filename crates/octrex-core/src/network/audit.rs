use super::decision::NetworkDecision;
use super::request::NetworkRequest;
use crate::db::DatabaseManager;
use crate::error::OctrexError;
use crate::events::{EventBus, EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAuditRecord {
    pub id: String,
    pub timestamp: u64,
    pub request_id: String,
    pub source: String,
    pub capability: String,
    pub destination: String,
    pub disposition: String,
    pub reason: String,
    pub matched_rule: Option<String>,
    pub policy_source: String,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
}

pub struct NetworkAuditLogger {
    db: Arc<DatabaseManager>,
    event_bus: Arc<EventBus>,
}

impl NetworkAuditLogger {
    pub fn new(db: Arc<DatabaseManager>, event_bus: Arc<EventBus>) -> Self {
        Self { db, event_bus }
    }

    pub fn log_decision(&self, request: &NetworkRequest, decision: &NetworkDecision) {
        let audit = NetworkAuditRecord {
            id: format!("netaudit-{}", uuid::Uuid::new_v4()),
            timestamp: decision.timestamp,
            request_id: request.request_id.to_string(),
            source: request.source.clone(),
            capability: request.capability.to_string(),
            destination: decision.endpoint.to_string(),
            disposition: format!("{:?}", decision.disposition),
            reason: decision.reason.clone(),
            matched_rule: decision.matched_rule.clone(),
            policy_source: decision.policy_source.to_string(),
            workspace_id: request
                .workspace_id
                .as_ref()
                .map(|w| w.as_str().to_string()),
            session_id: request.session_id.as_ref().map(|s| s.as_str().to_string()),
            task_id: request.task_id.as_ref().map(|t| t.as_str().to_string()),
            provider_id: request.provider_id.clone(),
            model_id: request.model_id.clone(),
        };

        // Emit security event to EventBus
        let event_type = if decision.is_allowed() {
            EventType::NetworkAllowed
        } else {
            EventType::NetworkBlocked
        };

        let event = EventEnvelope::new(
            event_type,
            serde_json::json!({
                "audit_id": audit.id,
                "request_id": audit.request_id,
                "capability": audit.capability,
                "destination": audit.destination,
                "disposition": audit.disposition,
                "reason": audit.reason,
                "policy_source": audit.policy_source,
                "provider_id": audit.provider_id,
            }),
        )
        .with_request_id(request.request_id.clone());

        let _ = self.event_bus.publish(event);

        // Record in SQLite database audit_records table
        let actor = format!("network_service:{}", request.source);
        let success = decision.is_allowed();

        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_records (
                    id, timestamp, event_type, task_id, session_id, workspace_id,
                    actor, provider, model, route, privacy_classification,
                    policy_source, permission, tool, success, reason
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    audit.id,
                    audit.timestamp as i64,
                    if success {
                        "NETWORK_ALLOWED"
                    } else {
                        "NETWORK_BLOCKED"
                    },
                    audit.task_id,
                    audit.session_id,
                    audit.workspace_id,
                    actor,
                    audit.provider_id,
                    audit.model_id,
                    audit.destination,
                    request.privacy_decision,
                    audit.policy_source,
                    audit.capability,
                    request.source,
                    if success { 1 } else { 0 },
                    audit.reason,
                ],
            )
            .map(|_| ())
            .map_err(|e| OctrexError::Internal {
                message: e.to_string(),
            })
        });
    }
}
