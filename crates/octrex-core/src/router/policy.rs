use crate::privacy::{
    DecisionState, PolicySource, PrivacyClassification, PrivacyContext, PrivacyGate, PrivacyMode,
};
use crate::providers::ExecutionMode;
use crate::router::request::RoutingRequest;
use crate::router::types::RoutingMode;
use std::sync::Arc;

pub struct PolicyEvaluator {
    privacy_gate: Arc<PrivacyGate>,
}

impl PolicyEvaluator {
    pub fn new(privacy_gate: Arc<PrivacyGate>) -> Self {
        Self { privacy_gate }
    }

    /// Evaluates if an execution mode is authorized under strict policy hierarchy
    pub fn evaluate_execution_mode(
        &self,
        req: &RoutingRequest,
        target_mode: ExecutionMode,
        workspace_classification: PrivacyClassification,
    ) -> (bool, DecisionState, PolicySource, String) {
        let user_privacy_mode = match req.routing_mode {
            RoutingMode::LocalOnly => PrivacyMode::LocalOnly,
            RoutingMode::OnlineOnly => PrivacyMode::OnlineOnly,
            RoutingMode::OnPremiseOnly => PrivacyMode::Auto,
            RoutingMode::ExplicitModel => PrivacyMode::Auto,
            RoutingMode::Auto => PrivacyMode::Auto,
        };

        // Formulate PrivacyContext for PrivacyGate
        let mut priv_ctx = PrivacyContext::new(req.request_id.clone());
        priv_ctx.task_id = req.task_id.clone();
        priv_ctx.session_id = req.session_id.clone();
        priv_ctx.workspace_id = req.workspace_id.clone();
        priv_ctx.workspace_classification = workspace_classification;
        priv_ctx.user_privacy_mode = user_privacy_mode;
        priv_ctx.inputs = req.inputs.clone();
        priv_ctx.requested_mode = target_mode;

        // Authoritative PrivacyGate evaluation
        let decision = self.privacy_gate.evaluate(&priv_ctx);

        let is_allowed = match target_mode {
            ExecutionMode::Local => decision
                .allowed_execution_modes
                .contains(&ExecutionMode::Local),
            ExecutionMode::OnPremise => decision
                .allowed_execution_modes
                .contains(&ExecutionMode::OnPremise),
            ExecutionMode::Cloud => {
                decision
                    .allowed_execution_modes
                    .contains(&ExecutionMode::Cloud)
                    && (decision.decision == DecisionState::AllowOnline
                        || decision.decision == DecisionState::AllowLocal)
            }
        };

        // Extra enforce: If User requested Online, but Company / Privacy Policy forbids online, fail closed!
        let mode_permitted_by_routing_req = match req.routing_mode {
            RoutingMode::LocalOnly => target_mode == ExecutionMode::Local,
            RoutingMode::OnPremiseOnly => target_mode == ExecutionMode::OnPremise,
            RoutingMode::OnlineOnly => target_mode == ExecutionMode::Cloud,
            RoutingMode::Auto | RoutingMode::ExplicitModel => true,
        };

        let effective_allowed = is_allowed && mode_permitted_by_routing_req;

        (
            effective_allowed,
            decision.decision,
            decision.selected_policy_source,
            decision.reason,
        )
    }
}
