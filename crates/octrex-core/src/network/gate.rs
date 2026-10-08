use super::decision::{Disposition, NetworkDecision, PolicySource};
use super::request::NetworkRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub execution_mode: String,
    pub privacy_decision: Option<String>,
    pub policy_version: String,
    pub consent_state: Option<String>,
    pub source: String,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            execution_mode: "local_only".to_string(),
            privacy_decision: None,
            policy_version: "1.0.0".to_string(),
            consent_state: None,
            source: "octrex_core".to_string(),
        }
    }
}

pub struct OutboundPayloadBoundary;

impl OutboundPayloadBoundary {
    /// Combines a Privacy Gate decision (from Phase 6) with a Network Security decision (Phase 7).
    /// Both security and privacy requirements MUST be satisfied for a request to proceed.
    pub fn evaluate_combined(
        privacy_disposition: Disposition,
        network_decision: NetworkDecision,
        request: &NetworkRequest,
    ) -> NetworkDecision {
        // If Privacy blocked, payload cannot leave machine
        if privacy_disposition.is_blocked() {
            return NetworkDecision::block(
                "Blocked by Privacy Gate: Payload classification prevents outbound transmission",
                None,
                PolicySource::Privacy,
                request.endpoint.clone(),
            );
        }

        // If Privacy requires consent and Network allowed, combined disposition is RequireConsent
        if privacy_disposition.requires_consent() && network_decision.is_allowed() {
            return NetworkDecision::require_consent(
                "Payload requires user consent before leaving machine",
                network_decision.matched_rule,
                PolicySource::Privacy,
                request.endpoint.clone(),
            );
        }

        // Otherwise return the network decision (which could be Block, Allow, or RequireConsent)
        network_decision
    }
}
