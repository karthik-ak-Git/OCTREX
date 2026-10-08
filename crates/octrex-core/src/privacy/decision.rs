use super::types::{DecisionState, PrivacyDecision};
use crate::db::models::AuditRecord;
use crate::events::types::PrivacyDecisionPayload;

pub struct DecisionFormatter;

impl DecisionFormatter {
    pub fn is_allowed(decision: &PrivacyDecision) -> bool {
        matches!(
            decision.decision,
            DecisionState::AllowLocal | DecisionState::AllowOnPremise | DecisionState::AllowOnline
        )
    }

    pub fn to_event_payload(decision: &PrivacyDecision) -> PrivacyDecisionPayload {
        PrivacyDecisionPayload {
            resource: format!(
                "ExecutionMode::{:?} / Classification::{}",
                decision.requested_execution_mode, decision.classification
            ),
            allowed: Self::is_allowed(decision),
            reason: decision.reason.clone(),
        }
    }

    pub fn to_audit_record(&self, decision: &PrivacyDecision) -> AuditRecord {
        AuditRecord {
            id: decision.id.clone(),
            timestamp: decision.timestamp,
            event_type: "PRIVACY_DECISION".to_string(),
            task_id: decision.task_id.clone(),
            session_id: decision.session_id.clone(),
            workspace_id: decision.workspace_id.clone(),
            actor: "PRIVACY_GATE".to_string(),
            provider: None,
            model: None,
            route: Some(format!("{:?}", decision.requested_execution_mode)),
            privacy_classification: Some(decision.classification.to_string()),
            policy_source: Some(decision.selected_policy_source.to_string()),
            permission: Some(decision.decision.to_string()),
            tool: None,
            success: Self::is_allowed(decision),
            reason: Some(decision.reason.clone()),
        }
    }
}
