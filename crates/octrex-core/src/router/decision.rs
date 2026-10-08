use crate::hardware::CompatibilityResult;
use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use crate::models::types::ModelDescriptor;
use crate::router::types::{RoutingDecision, RoutingDecisionState, RoutingEvidence};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RoutingDecisionBuilder;

impl RoutingDecisionBuilder {
    pub fn build_selected(
        request_id: RequestId,
        task_id: Option<TaskId>,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
        descriptor: ModelDescriptor,
        reason: impl Into<String>,
        confidence: f64,
        evidence: RoutingEvidence,
        policy_evidence: Vec<String>,
        hardware_compatibility: Option<CompatibilityResult>,
        context_compatibility: Option<String>,
    ) -> RoutingDecision {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        RoutingDecision {
            id: format!("route-{}", uuid::Uuid::new_v4().simple()),
            request_id,
            task_id,
            session_id,
            workspace_id,
            selected_model_id: Some(descriptor.id.clone()),
            selected_provider_id: Some(descriptor.provider_id.clone()),
            execution_mode: Some(descriptor.execution_mode),
            selected_descriptor: Some(descriptor),
            state: RoutingDecisionState::Selected,
            reason: reason.into(),
            confidence,
            timestamp: now,
            evidence,
            policy_evidence,
            hardware_compatibility,
            context_compatibility,
        }
    }

    pub fn build_rejected(
        request_id: RequestId,
        task_id: Option<TaskId>,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
        state: RoutingDecisionState,
        reason: impl Into<String>,
        evidence: RoutingEvidence,
        policy_evidence: Vec<String>,
    ) -> RoutingDecision {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        RoutingDecision {
            id: format!("route-{}", uuid::Uuid::new_v4().simple()),
            request_id,
            task_id,
            session_id,
            workspace_id,
            selected_model_id: None,
            selected_provider_id: None,
            execution_mode: None,
            selected_descriptor: None,
            state,
            reason: reason.into(),
            confidence: 1.0,
            timestamp: now,
            evidence,
            policy_evidence,
            hardware_compatibility: None,
            context_compatibility: None,
        }
    }
}
