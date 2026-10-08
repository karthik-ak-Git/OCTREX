use crate::db::manager::DatabaseManager;
use crate::events::EventBus;
use crate::hardware::{CompatibilityStatus, HardwareService, ModelCompatibilityEngine};
use crate::ids::WorkspaceId;
use crate::models::registry::ModelRegistry;
use crate::models::types::ModelDescriptor;
use crate::privacy::{PrivacyClassification, PrivacyGate};
use crate::providers::registry::ProviderRegistry;
use crate::providers::ExecutionMode;
use crate::router::audit::RouterAuditLogger;
use crate::router::candidates::CandidatePipeline;
use crate::router::decision::RoutingDecisionBuilder;
use crate::router::events::RouterEventNotifier;
use crate::router::request::RoutingRequest;
use crate::router::types::{
    CandidateEliminationReason, RoutingDecision, RoutingDecisionState, RoutingEvidence,
};
use std::sync::Arc;

pub struct ModelRouter {
    model_registry: Arc<ModelRegistry>,
    provider_registry: Arc<ProviderRegistry>,
    hardware_service: Arc<HardwareService>,
    candidate_pipeline: CandidatePipeline,
    event_notifier: RouterEventNotifier,
    audit_logger: RouterAuditLogger,
}

impl ModelRouter {
    pub fn new(
        model_registry: Arc<ModelRegistry>,
        provider_registry: Arc<ProviderRegistry>,
        privacy_gate: Arc<PrivacyGate>,
        hardware_service: Arc<HardwareService>,
        event_bus: Arc<EventBus>,
        db: Arc<DatabaseManager>,
    ) -> Self {
        let candidate_pipeline = CandidatePipeline::new(
            model_registry.clone(),
            provider_registry.clone(),
            privacy_gate.clone(),
            hardware_service.clone(),
        );

        let event_notifier = RouterEventNotifier::new(event_bus);
        let audit_logger = RouterAuditLogger::new(db.clone());

        Self {
            model_registry,
            provider_registry,
            hardware_service,
            candidate_pipeline,
            event_notifier,
            audit_logger,
        }
    }

    /// Evaluates a RoutingRequest and produces an authoritative RoutingDecision.
    pub fn route(&self, req: RoutingRequest) -> RoutingDecision {
        self.event_notifier.notify_started(&req);

        let workspace_classification = req
            .privacy_classification
            .unwrap_or(PrivacyClassification::Public);

        let (evaluations, eligible_models) = self
            .candidate_pipeline
            .evaluate_candidates(&req, workspace_classification);

        let candidate_count = evaluations.len();
        let eligible_count = eligible_models.len();

        let evidence = RoutingEvidence {
            candidate_count,
            eligible_count,
            excluded_candidates: evaluations
                .iter()
                .filter(|e| !e.eligible)
                .cloned()
                .collect(),
            selected_candidate: None,
            privacy_status: format!("Workspace Classification: {}", workspace_classification),
            hardware_status: format!(
                "Hardware profile active (Confidence: {:?})",
                self.hardware_service.get_profile().confidence
            ),
            context_status: format!("Required input context: {:?}", req.required_context_tokens),
            provider_status: format!(
                "Registered providers: {}",
                self.provider_registry.list_providers().len()
            ),
            policy_status: "Authoritative Privacy Gate + Policy Engine active".to_string(),
        };

        let mut policy_evidence = vec![
            format!("Privacy Classification: {}", workspace_classification),
            format!("Routing Mode: {}", req.routing_mode),
        ];

        // 1. If eligible candidates exist, pick the top-scored candidate
        if let Some(selected_descriptor) = eligible_models.first().cloned() {
            let selected_eval = evaluations
                .iter()
                .find(|e| e.model_id == selected_descriptor.id)
                .cloned();

            let mut final_evidence = evidence.clone();
            final_evidence.selected_candidate = selected_eval;

            let profile = self.hardware_service.get_profile();
            let hw_compat = ModelCompatibilityEngine::evaluate(&profile, &selected_descriptor);

            let reason = format!(
                "Selected model '{}' ({}) via {} routing pipeline (Score: {:.1})",
                selected_descriptor.display_name,
                selected_descriptor.execution_mode,
                req.routing_mode,
                final_evidence
                    .selected_candidate
                    .as_ref()
                    .map(|c| c.score)
                    .unwrap_or(100.0)
            );

            policy_evidence.push(format!("Selected Model ID: {}", selected_descriptor.id));
            policy_evidence.push(format!(
                "Execution Mode: {}",
                selected_descriptor.execution_mode
            ));

            let decision = RoutingDecisionBuilder::build_selected(
                req.request_id,
                req.task_id,
                req.session_id,
                req.workspace_id,
                selected_descriptor,
                reason,
                1.0,
                final_evidence,
                policy_evidence,
                Some(hw_compat),
                Some("Context requirement satisfied".to_string()),
            );

            self.event_notifier.notify_decision(&decision);
            self.audit_logger.log_decision(&decision);
            return decision;
        }

        // 2. If no eligible candidates, formulate clear rejection decision state
        let (state, reason) = self.determine_rejection_reason(&evaluations, candidate_count);
        policy_evidence.push(format!("Rejection State: {}", state));
        policy_evidence.push(format!("Rejection Reason: {}", reason));

        let decision = RoutingDecisionBuilder::build_rejected(
            req.request_id,
            req.task_id,
            req.session_id,
            req.workspace_id,
            state,
            reason,
            evidence,
            policy_evidence,
        );

        self.event_notifier.notify_decision(&decision);
        self.audit_logger.log_decision(&decision);
        decision
    }

    /// Preview routing without side-effects, audit logging, or network calls
    pub fn preview(&self, req: RoutingRequest) -> RoutingDecision {
        let workspace_classification = req
            .privacy_classification
            .unwrap_or(PrivacyClassification::Public);

        let (evaluations, eligible_models) = self
            .candidate_pipeline
            .evaluate_candidates(&req, workspace_classification);

        let evidence = RoutingEvidence {
            candidate_count: evaluations.len(),
            eligible_count: eligible_models.len(),
            excluded_candidates: evaluations
                .iter()
                .filter(|e| !e.eligible)
                .cloned()
                .collect(),
            selected_candidate: None,
            privacy_status: format!("Workspace Classification: {}", workspace_classification),
            hardware_status: "Diagnostic Hardware Profile".to_string(),
            context_status: format!("Required context: {:?}", req.required_context_tokens),
            provider_status: "Provider Registry Diagnostic".to_string(),
            policy_status: "Preview Policy Evaluator".to_string(),
        };

        if let Some(selected) = eligible_models.first().cloned() {
            let profile = self.hardware_service.get_profile();
            let hw_compat = ModelCompatibilityEngine::evaluate(&profile, &selected);
            return RoutingDecisionBuilder::build_selected(
                req.request_id,
                req.task_id,
                req.session_id,
                req.workspace_id,
                selected,
                "Preview evaluation successful",
                1.0,
                evidence,
                vec!["Preview evaluation".to_string()],
                Some(hw_compat),
                Some("Context window capacity check passed".to_string()),
            );
        }

        let (state, reason) = self.determine_rejection_reason(&evaluations, evaluations.len());
        RoutingDecisionBuilder::build_rejected(
            req.request_id,
            req.task_id,
            req.session_id,
            req.workspace_id,
            state,
            reason,
            evidence,
            vec!["Preview evaluation rejected".to_string()],
        )
    }

    pub fn get_compatible_models(
        &self,
        _workspace_id: Option<&WorkspaceId>,
    ) -> Vec<ModelDescriptor> {
        let profile = self.hardware_service.get_profile();
        self.model_registry
            .list_models()
            .into_iter()
            .filter(|m| {
                if m.execution_mode == ExecutionMode::Cloud {
                    return true;
                }
                let res = ModelCompatibilityEngine::evaluate(&profile, m);
                res.status != CompatibilityStatus::Incompatible
            })
            .collect()
    }

    pub fn get_recommended_models(&self, purpose: Option<&str>) -> Vec<ModelDescriptor> {
        let req = RoutingRequest::new(purpose.unwrap_or("general"));
        let (_, eligible) = self
            .candidate_pipeline
            .evaluate_candidates(&req, PrivacyClassification::Public);
        eligible
    }

    fn determine_rejection_reason(
        &self,
        evaluations: &[crate::router::types::CandidateEvaluation],
        candidate_count: usize,
    ) -> (RoutingDecisionState, String) {
        if candidate_count == 0 {
            return (
                RoutingDecisionState::NoCompatibleModel,
                "No models are currently registered in ModelRegistry".to_string(),
            );
        }

        let mut policy_denied = false;
        let mut hw_denied = false;
        let mut ctx_denied = false;

        for eval in evaluations {
            if let Some(reason) = &eval.elimination_reason {
                match reason {
                    CandidateEliminationReason::PolicyDenied { .. }
                    | CandidateEliminationReason::PrivacyDenied { .. } => policy_denied = true,
                    CandidateEliminationReason::HardwareIncompatible { .. } => hw_denied = true,
                    CandidateEliminationReason::ContextTooSmall { .. } => ctx_denied = true,
                    _ => {}
                }
            }
        }

        if policy_denied {
            (
                RoutingDecisionState::PolicyDenied,
                "Candidate models were blocked by workspace or company privacy policy".to_string(),
            )
        } else if hw_denied {
            (
                RoutingDecisionState::HardwareIncompatible,
                "Available local models do not meet system hardware requirements (RAM/VRAM/GPU)"
                    .to_string(),
            )
        } else if ctx_denied {
            (
                RoutingDecisionState::ContextTooLarge,
                "Required task context length exceeds available model context windows".to_string(),
            )
        } else {
            (
                RoutingDecisionState::NoCompatibleModel,
                "No candidate model satisfied all task and security constraints".to_string(),
            )
        }
    }
}
