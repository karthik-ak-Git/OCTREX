use super::classifier::PrivacyClassifier;
use super::consent::ConsentManager;
use super::policy::PolicyEngine;
use super::types::{
    ClassificationConfidence, DecisionState, EffectivePrivacyStatus, PolicyAction, PolicyRule,
    PolicySource, PrivacyClassification, PrivacyContext, PrivacyDecision, PrivacyMode,
};
use crate::providers::ExecutionMode;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PrivacyGate {
    classifier: PrivacyClassifier,
    policy_engine: Arc<PolicyEngine>,
    consent_manager: Arc<ConsentManager>,
}

impl PrivacyGate {
    pub fn new() -> Self {
        Self {
            classifier: PrivacyClassifier::new(),
            policy_engine: Arc::new(PolicyEngine::new()),
            consent_manager: Arc::new(ConsentManager::new()),
        }
    }

    pub fn with_components(
        policy_engine: Arc<PolicyEngine>,
        consent_manager: Arc<ConsentManager>,
    ) -> Self {
        Self {
            classifier: PrivacyClassifier::new(),
            policy_engine,
            consent_manager,
        }
    }

    pub fn policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    pub fn consent_manager(&self) -> &ConsentManager {
        &self.consent_manager
    }

    /// Primary Privacy Gate boundary evaluation function.
    /// Evaluates privacy context deterministically without LLM calls or network execution.
    pub fn evaluate(&self, context: &PrivacyContext) -> PrivacyDecision {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // 1. Run Data Classifier over inputs & workspace
        let class_result = self
            .classifier
            .classify(context.workspace_classification, &context.inputs);

        let effective_classification = class_result.classification;

        // 2. Evaluate Policy Engine across hierarchy (System > Company > Security > Privacy > User)
        let (policy_action, winning_rule, selected_source) = self
            .policy_engine
            .evaluate(context, effective_classification);

        // 3. Compute Allowed Execution Modes
        let mut allowed_modes = vec![ExecutionMode::Local, ExecutionMode::OnPremise];

        let cloud_allowed_by_policy = match policy_action {
            PolicyAction::Allow => true,
            PolicyAction::RequireConsent => context.consent_granted,
            PolicyAction::Deny | PolicyAction::RequireReview => false,
        };

        // Enforce hard security bounds for Cloud execution
        let hard_cloud_blocked = context.confidential_mode_enabled
            || context.user_privacy_mode == PrivacyMode::Confidential
            || context.user_privacy_mode == PrivacyMode::LocalOnly
            || effective_classification >= PrivacyClassification::Confidential
            || context.workspace_classification >= PrivacyClassification::Confidential;

        if cloud_allowed_by_policy && !hard_cloud_blocked {
            allowed_modes.push(ExecutionMode::Cloud);
        }

        // 4. Formulate Decision State
        let decision_state = match context.requested_mode {
            ExecutionMode::Local => DecisionState::AllowLocal,
            ExecutionMode::OnPremise => DecisionState::AllowOnPremise,
            ExecutionMode::Cloud => {
                if allowed_modes.contains(&ExecutionMode::Cloud) {
                    DecisionState::AllowOnline
                } else if policy_action == PolicyAction::RequireConsent && !context.consent_granted
                {
                    DecisionState::RequireConsent
                } else if policy_action == PolicyAction::RequireReview {
                    DecisionState::RequireReview
                } else if hard_cloud_blocked
                    || winning_rule.source >= PolicySource::Company
                    || policy_action == PolicyAction::Deny
                {
                    if context.user_privacy_mode == PrivacyMode::OnlineOnly {
                        DecisionState::Deny
                    } else {
                        DecisionState::DenyOnline
                    }
                } else {
                    DecisionState::DenyOnline
                }
            }
        };

        let decision_id = format!("priv-{}", uuid::Uuid::new_v4().simple());

        PrivacyDecision {
            id: decision_id,
            request_id: context.request_id.clone(),
            session_id: context.session_id.clone(),
            task_id: context.task_id.clone(),
            workspace_id: context.workspace_id.clone(),
            classification: effective_classification,
            requested_execution_mode: context.requested_mode,
            allowed_execution_modes: allowed_modes,
            selected_policy_source: selected_source,
            policy_version: winning_rule.version,
            decision: decision_state,
            reason: winning_rule.reason,
            confidence: class_result.confidence,
            timestamp: now,
            evidence_summary: class_result.evidence_summary,
        }
    }

    /// Retrieves current effective privacy status summary
    pub fn get_effective_status(
        &self,
        current_mode: PrivacyMode,
        confidential_mode: bool,
        workspace_id: Option<String>,
        workspace_classification: PrivacyClassification,
    ) -> EffectivePrivacyStatus {
        let rules = self.policy_engine.list_rules();
        EffectivePrivacyStatus {
            privacy_mode: current_mode,
            confidential_mode,
            active_workspace_id: workspace_id,
            active_workspace_classification: workspace_classification,
            active_policies_count: rules.len(),
            system_policy_status: "ACTIVE (System priority 100)".to_string(),
            company_policy_status: "ACTIVE (Company priority 80)".to_string(),
            default_mode: PrivacyMode::LocalOnly,
        }
    }
}

impl Default for PrivacyGate {
    fn default() -> Self {
        Self::new()
    }
}
