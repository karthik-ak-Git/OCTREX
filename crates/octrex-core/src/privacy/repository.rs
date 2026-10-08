use super::types::{ConsentDecision, ConsentRequest, PrivacyClassification, PrivacyDecision};
use crate::db::manager::DatabaseManager;
use crate::error::OctrexError;
use crate::ids::WorkspaceId;
use rusqlite::params;

pub trait PrivacyRepository: Send + Sync {
    fn save_decision(&self, decision: &PrivacyDecision) -> Result<(), OctrexError>;
    fn get_decision(&self, id: &str) -> Result<Option<PrivacyDecision>, OctrexError>;
    fn list_recent_decisions(&self, limit: usize) -> Result<Vec<PrivacyDecision>, OctrexError>;

    fn save_consent_request(&self, req: &ConsentRequest) -> Result<(), OctrexError>;
    fn record_consent_decision(&self, dec: &ConsentDecision) -> Result<(), OctrexError>;

    fn get_workspace_classification(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<PrivacyClassification, OctrexError>;
    fn set_workspace_classification(
        &self,
        workspace_id: &WorkspaceId,
        classification: PrivacyClassification,
    ) -> Result<(), OctrexError>;
}

pub struct SqlitePrivacyRepository {
    db: DatabaseManager,
}

impl SqlitePrivacyRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl PrivacyRepository for SqlitePrivacyRepository {
    fn save_decision(&self, decision: &PrivacyDecision) -> Result<(), OctrexError> {
        self.db.with_conn(|conn| {
            let task_id_str = decision.task_id.as_ref().map(|t| t.as_str());
            let sess_id_str = decision.session_id.as_ref().map(|s| s.as_str());
            let ws_id_str = decision.workspace_id.as_ref().map(|w| w.as_str());
            let allowed_modes_json = serde_json::to_string(&decision.allowed_execution_modes).unwrap_or_default();

            conn.execute(
                "INSERT INTO privacy_decisions (id, request_id, task_id, session_id, workspace_id, classification, requested_mode, allowed_modes_json, decision, policy_source, policy_version, reason, confidence, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    decision.id,
                    decision.request_id.as_str(),
                    task_id_str,
                    sess_id_str,
                    ws_id_str,
                    decision.classification.to_string(),
                    format!("{:?}", decision.requested_execution_mode),
                    allowed_modes_json,
                    decision.decision.to_string(),
                    decision.selected_policy_source.to_string(),
                    decision.policy_version,
                    decision.reason,
                    format!("{:?}", decision.confidence),
                    decision.timestamp
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to save privacy decision to SQLite: {}", e),
            })?;
            Ok(())
        })
    }

    fn get_decision(&self, id: &str) -> Result<Option<PrivacyDecision>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, request_id, task_id, session_id, workspace_id, classification, requested_mode, allowed_modes_json, decision, policy_source, policy_version, reason, confidence, timestamp FROM privacy_decisions WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let req_id_str: String = row.get(1)?;
                let task_id_str: Option<String> = row.get(2)?;
                let sess_id_str: Option<String> = row.get(3)?;
                let ws_id_str: Option<String> = row.get(4)?;
                let class_str: String = row.get(5)?;
                let req_mode_str: String = row.get(6)?;
                let allowed_modes_json: String = row.get(7)?;
                let decision_str: String = row.get(8)?;
                let policy_src_str: String = row.get(9)?;
                let policy_ver: u32 = row.get(10)?;
                let reason: String = row.get(11)?;
                let conf_str: String = row.get(12)?;
                let timestamp: u64 = row.get(13)?;

                let classification = class_str.parse().unwrap_or(PrivacyClassification::Public);
                let selected_policy_source = policy_src_str.parse().unwrap_or(super::types::PolicySource::System);
                let allowed_execution_modes: Vec<crate::providers::ExecutionMode> = serde_json::from_str(&allowed_modes_json).unwrap_or_default();
                let requested_execution_mode = match req_mode_str.to_lowercase().as_str() {
                    "cloud" | "online" => crate::providers::ExecutionMode::Cloud,
                    "onpremise" | "on_premise" => crate::providers::ExecutionMode::OnPremise,
                    _ => crate::providers::ExecutionMode::Local,
                };
                let decision = match decision_str.as_str() {
                    "ALLOW_LOCAL" => super::types::DecisionState::AllowLocal,
                    "ALLOW_ON_PREMISE" => super::types::DecisionState::AllowOnPremise,
                    "ALLOW_ONLINE" => super::types::DecisionState::AllowOnline,
                    "DENY_ONLINE" => super::types::DecisionState::DenyOnline,
                    "REQUIRE_CONSENT" => super::types::DecisionState::RequireConsent,
                    "REQUIRE_REVIEW" => super::types::DecisionState::RequireReview,
                    _ => super::types::DecisionState::Deny,
                };
                let confidence = match conf_str.as_str() {
                    "High" => super::types::ClassificationConfidence::High,
                    "Medium" => super::types::ClassificationConfidence::Medium,
                    _ => super::types::ClassificationConfidence::Low,
                };

                Ok(PrivacyDecision {
                    id: id_str,
                    request_id: crate::ids::RequestId::from(req_id_str),
                    session_id: sess_id_str.map(crate::ids::SessionId::from),
                    task_id: task_id_str.map(crate::ids::TaskId::from),
                    workspace_id: ws_id_str.map(crate::ids::WorkspaceId::from),
                    classification,
                    requested_execution_mode,
                    allowed_execution_modes,
                    selected_policy_source,
                    policy_version: policy_ver,
                    decision,
                    reason,
                    confidence,
                    timestamp,
                    evidence_summary: "Loaded from SQLite DB".to_string(),
                })
            });

            match result {
                Ok(d) => Ok(Some(d)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_recent_decisions(&self, limit: usize) -> Result<Vec<PrivacyDecision>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, request_id, task_id, session_id, workspace_id, classification, requested_mode, allowed_modes_json, decision, policy_source, policy_version, reason, confidence, timestamp FROM privacy_decisions ORDER BY timestamp DESC LIMIT ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![limit as i64], |row| {
                    let id_str: String = row.get(0)?;
                    let req_id_str: String = row.get(1)?;
                    let task_id_str: Option<String> = row.get(2)?;
                    let sess_id_str: Option<String> = row.get(3)?;
                    let ws_id_str: Option<String> = row.get(4)?;
                    let class_str: String = row.get(5)?;
                    let req_mode_str: String = row.get(6)?;
                    let allowed_modes_json: String = row.get(7)?;
                    let decision_str: String = row.get(8)?;
                    let policy_src_str: String = row.get(9)?;
                    let policy_ver: u32 = row.get(10)?;
                    let reason: String = row.get(11)?;
                    let conf_str: String = row.get(12)?;
                    let timestamp: u64 = row.get(13)?;

                    let classification = class_str.parse().unwrap_or(PrivacyClassification::Public);
                    let selected_policy_source = policy_src_str.parse().unwrap_or(super::types::PolicySource::System);
                    let allowed_execution_modes: Vec<crate::providers::ExecutionMode> = serde_json::from_str(&allowed_modes_json).unwrap_or_default();
                    let requested_execution_mode = match req_mode_str.to_lowercase().as_str() {
                        "cloud" | "online" => crate::providers::ExecutionMode::Cloud,
                        "onpremise" | "on_premise" => crate::providers::ExecutionMode::OnPremise,
                        _ => crate::providers::ExecutionMode::Local,
                    };
                    let decision = match decision_str.as_str() {
                        "ALLOW_LOCAL" => super::types::DecisionState::AllowLocal,
                        "ALLOW_ON_PREMISE" => super::types::DecisionState::AllowOnPremise,
                        "ALLOW_ONLINE" => super::types::DecisionState::AllowOnline,
                        "DENY_ONLINE" => super::types::DecisionState::DenyOnline,
                        "REQUIRE_CONSENT" => super::types::DecisionState::RequireConsent,
                        "REQUIRE_REVIEW" => super::types::DecisionState::RequireReview,
                        _ => super::types::DecisionState::Deny,
                    };
                    let confidence = match conf_str.as_str() {
                        "High" => super::types::ClassificationConfidence::High,
                        "Medium" => super::types::ClassificationConfidence::Medium,
                        _ => super::types::ClassificationConfidence::Low,
                    };

                    Ok(PrivacyDecision {
                        id: id_str,
                        request_id: crate::ids::RequestId::from(req_id_str),
                        session_id: sess_id_str.map(crate::ids::SessionId::from),
                        task_id: task_id_str.map(crate::ids::TaskId::from),
                        workspace_id: ws_id_str.map(crate::ids::WorkspaceId::from),
                        classification,
                        requested_execution_mode,
                        allowed_execution_modes,
                        selected_policy_source,
                        policy_version: policy_ver,
                        decision,
                        reason,
                        confidence,
                        timestamp,
                        evidence_summary: "Loaded from SQLite DB".to_string(),
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

    fn save_consent_request(&self, req: &ConsentRequest) -> Result<(), OctrexError> {
        self.db.with_conn(|conn| {
            let task_id_str = req.task_id.as_ref().map(|t| t.as_str());
            let sess_id_str = req.session_id.as_ref().map(|s| s.as_str());
            let ws_id_str = req.workspace_id.as_ref().map(|w| w.as_str());

            conn.execute(
                "INSERT INTO privacy_consents (id, request_id, task_id, session_id, workspace_id, data_classification, destination_provider, destination_model, requested_mode, granted, reason, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    req.id,
                    req.request_id.as_str(),
                    task_id_str,
                    sess_id_str,
                    ws_id_str,
                    req.classification.to_string(),
                    req.destination_provider,
                    req.destination_model,
                    format!("{:?}", req.requested_mode),
                    0, // pending
                    req.reasoning,
                    req.created_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record consent request: {}", e),
            })?;
            Ok(())
        })
    }

    fn record_consent_decision(&self, dec: &ConsentDecision) -> Result<(), OctrexError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE privacy_consents SET granted = ?1, reason = ?2 WHERE id = ?3",
                params![if dec.granted { 1 } else { 0 }, dec.reason, dec.consent_id],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to update consent decision: {}", e),
            })?;
            Ok(())
        })
    }

    fn get_workspace_classification(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<PrivacyClassification, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT classification FROM workspaces WHERE id = ?1")
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;

            let result = stmt.query_row(params![workspace_id.as_str()], |row| {
                let class_str: Option<String> = row.get(0).ok();
                Ok(class_str)
            });

            match result {
                Ok(Some(s)) => Ok(s.parse().unwrap_or(PrivacyClassification::Public)),
                _ => Ok(PrivacyClassification::Public),
            }
        })
    }

    fn set_workspace_classification(
        &self,
        workspace_id: &WorkspaceId,
        classification: PrivacyClassification,
    ) -> Result<(), OctrexError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE workspaces SET classification = ?1 WHERE id = ?2",
                params![classification.to_string(), workspace_id.as_str()],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to update workspace classification: {}", e),
            })?;
            Ok(())
        })
    }
}
