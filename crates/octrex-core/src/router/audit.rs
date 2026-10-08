use crate::db::manager::DatabaseManager;
use crate::router::types::RoutingDecision;
use rusqlite::params;
use std::sync::Arc;
use tracing::info;

pub struct RouterAuditLogger {
    db: Arc<DatabaseManager>,
}

impl RouterAuditLogger {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn log_decision(&self, decision: &RoutingDecision) {
        info!(
            decision_id = %decision.id,
            request_id = %decision.request_id.as_str(),
            state = %decision.state,
            selected_model = ?decision.selected_model_id,
            execution_mode = ?decision.execution_mode,
            reason = %decision.reason,
            "Auditing model routing decision"
        );

        let db_clone = self.db.clone();
        let dec_id = decision.id.clone();
        let req_id = decision.request_id.as_str().to_string();
        let task_id = decision.task_id.as_ref().map(|t| t.as_str().to_string());
        let session_id = decision.session_id.as_ref().map(|s| s.as_str().to_string());
        let workspace_id = decision
            .workspace_id
            .as_ref()
            .map(|w| w.as_str().to_string());
        let sel_model = decision.selected_model_id.clone();
        let sel_provider = decision.selected_provider_id.clone();
        let mode_str = decision
            .execution_mode
            .map(|m| m.to_string())
            .unwrap_or_else(|| "none".to_string());
        let state_str = decision.state.to_string();
        let reason_str = decision.reason.clone();
        let cand_cnt = decision.evidence.candidate_count as i64;
        let elig_cnt = decision.evidence.eligible_count as i64;
        let evidence_json = serde_json::to_string(&decision.evidence).unwrap_or_default();
        let timestamp = decision.timestamp as i64;

        let _ = db_clone.with_conn(move |conn| {
            let _ = conn.execute(
                "INSERT INTO routing_decisions (
                    id, request_id, task_id, session_id, workspace_id, purpose, user_privacy_mode,
                    selected_model, selected_provider, execution_mode, decision_state, reason,
                    candidate_count, eligible_count, evidence_json, timestamp
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    dec_id,
                    req_id,
                    task_id,
                    session_id,
                    workspace_id,
                    "model_routing",
                    "auto",
                    sel_model,
                    sel_provider,
                    mode_str,
                    state_str,
                    reason_str,
                    cand_cnt,
                    elig_cnt,
                    evidence_json,
                    timestamp,
                ],
            );
            Ok(())
        });
    }
}
