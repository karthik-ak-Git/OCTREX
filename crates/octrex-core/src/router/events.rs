use crate::events::{EventBus, EventEnvelope, EventType};
use crate::router::request::RoutingRequest;
use crate::router::types::{RoutingDecision, RoutingDecisionState};
use std::sync::Arc;

pub struct RouterEventNotifier {
    event_bus: Arc<EventBus>,
}

impl RouterEventNotifier {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    pub fn notify_started(&self, req: &RoutingRequest) {
        let envelope = EventEnvelope::new(
            EventType::RoutingStarted,
            serde_json::json!({
                "purpose": req.purpose,
                "routing_mode": req.routing_mode.to_string(),
                "user_selected_model": req.user_selected_model,
                "allow_local": req.allow_local,
                "allow_on_prem": req.allow_on_prem,
                "allow_online": req.allow_online,
            }),
        )
        .with_request_id(req.request_id.clone());

        let _ = self.event_bus.publish(envelope);
    }

    pub fn notify_decision(&self, decision: &RoutingDecision) {
        let event_type = match decision.state {
            RoutingDecisionState::Selected => EventType::RoutingModelSelected,
            RoutingDecisionState::Blocked | RoutingDecisionState::PolicyDenied => {
                EventType::RoutingBlocked
            }
            RoutingDecisionState::NoCompatibleModel
            | RoutingDecisionState::ContextTooLarge
            | RoutingDecisionState::HardwareIncompatible => EventType::RoutingNoCompatibleModel,
            RoutingDecisionState::RequireUserSelection
            | RoutingDecisionState::NoAuthorizedModel => EventType::RoutingRequireUserSelection,
            RoutingDecisionState::ProviderUnavailable | RoutingDecisionState::Unknown => {
                EventType::RoutingBlocked
            }
        };

        let payload = serde_json::json!({
            "decision_id": decision.id,
            "decision_state": decision.state.to_string(),
            "selected_model_id": decision.selected_model_id,
            "selected_provider_id": decision.selected_provider_id,
            "execution_mode": decision.execution_mode.map(|m| m.to_string()),
            "reason": decision.reason,
            "candidate_count": decision.evidence.candidate_count,
            "eligible_count": decision.evidence.eligible_count,
            "confidence": decision.confidence,
        });

        let mut envelope =
            EventEnvelope::new(event_type, payload).with_request_id(decision.request_id.clone());

        if let Some(tid) = &decision.task_id {
            envelope = envelope.with_task_id(tid.clone());
        }
        if let Some(sid) = &decision.session_id {
            envelope = envelope.with_session_id(sid.clone());
        }
        if let Some(wid) = &decision.workspace_id {
            envelope = envelope.with_workspace_id(wid.clone());
        }

        let _ = self.event_bus.publish(envelope);
    }
}
