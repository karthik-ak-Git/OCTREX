use super::evidence::EvidenceManager;
use super::types::{
    ClassificationResult, ConsentDecision, ConsentRequest, OutboundPayloadPreview,
    PrivacyClassification, PrivacyContext,
};
use crate::error::OctrexError;
use crate::ids::RequestId;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ConsentManager {
    pending_requests: RwLock<HashMap<String, ConsentRequest>>,
    decisions_history: RwLock<Vec<ConsentDecision>>,
}

impl ConsentManager {
    pub fn new() -> Self {
        Self {
            pending_requests: RwLock::new(HashMap::new()),
            decisions_history: RwLock::new(Vec::new()),
        }
    }

    pub fn create_request(
        &self,
        context: &PrivacyContext,
        classification: PrivacyClassification,
        class_result: &ClassificationResult,
        reasoning: impl Into<String>,
    ) -> ConsentRequest {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let consent_id = format!("consent-{}", uuid::Uuid::new_v4().simple());
        let mut preview =
            EvidenceManager::build_payload_preview(context, classification, class_result);
        preview.policy_decision = super::types::DecisionState::RequireConsent;

        let req = ConsentRequest {
            id: consent_id.clone(),
            request_id: context.request_id.clone(),
            task_id: context.task_id.clone(),
            session_id: context.session_id.clone(),
            workspace_id: context.workspace_id.clone(),
            classification,
            destination_provider: context
                .candidate_provider
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            destination_model: context
                .candidate_model
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            requested_mode: context.requested_mode,
            payload_preview: preview,
            reasoning: reasoning.into(),
            created_at: now,
        };

        let mut guard = self.pending_requests.write().unwrap();
        guard.insert(consent_id.clone(), req.clone());
        req
    }

    pub fn process_decision(
        &self,
        consent_id: &str,
        granted: bool,
        user_reason: Option<String>,
    ) -> Result<ConsentDecision, OctrexError> {
        let req = {
            let mut guard = self.pending_requests.write().unwrap();
            guard
                .remove(consent_id)
                .ok_or_else(|| OctrexError::NotFound {
                    resource: format!("Consent request '{}'", consent_id),
                })?
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let dec = ConsentDecision {
            id: format!("dec-{}", uuid::Uuid::new_v4().simple()),
            request_id: req.request_id,
            consent_id: consent_id.to_string(),
            granted,
            reason: user_reason,
            timestamp: now,
        };

        let mut hist = self.decisions_history.write().unwrap();
        hist.push(dec.clone());
        Ok(dec)
    }

    pub fn get_pending_request(&self, consent_id: &str) -> Option<ConsentRequest> {
        let guard = self.pending_requests.read().unwrap();
        guard.get(consent_id).cloned()
    }

    pub fn list_pending_requests(&self) -> Vec<ConsentRequest> {
        let guard = self.pending_requests.read().unwrap();
        guard.values().cloned().collect()
    }
}

impl Default for ConsentManager {
    fn default() -> Self {
        Self::new()
    }
}
