use crate::hardware::CompatibilityStatus;
use crate::models::registry::ModelRegistry;
use crate::models::types::ModelDescriptor;
use crate::privacy::{PrivacyClassification, PrivacyGate};
use crate::providers::registry::ProviderRegistry;
use crate::providers::ExecutionMode;
use crate::router::constraints::CapabilityEvaluator;
use crate::router::context::ContextEvaluator;
use crate::router::hardware::HardwareEvaluator;
use crate::router::health::HealthEvaluator;
use crate::router::policy::PolicyEvaluator;
use crate::router::request::RoutingRequest;
use crate::router::scorer::ModelScorer;
use crate::router::types::{CandidateEliminationReason, CandidateEvaluation, RoutingMode};
use std::sync::Arc;

pub struct CandidatePipeline {
    model_registry: Arc<ModelRegistry>,
    health_evaluator: HealthEvaluator,
    policy_evaluator: PolicyEvaluator,
    hardware_evaluator: HardwareEvaluator,
}

impl CandidatePipeline {
    pub fn new(
        model_registry: Arc<ModelRegistry>,
        provider_registry: Arc<ProviderRegistry>,
        privacy_gate: Arc<PrivacyGate>,
        hardware_service: Arc<crate::hardware::HardwareService>,
    ) -> Self {
        Self {
            model_registry,
            health_evaluator: HealthEvaluator::new(provider_registry),
            policy_evaluator: PolicyEvaluator::new(privacy_gate),
            hardware_evaluator: HardwareEvaluator::new(hardware_service),
        }
    }

    pub fn evaluate_candidates(
        &self,
        req: &RoutingRequest,
        workspace_classification: PrivacyClassification,
    ) -> (Vec<CandidateEvaluation>, Vec<ModelDescriptor>) {
        let all_models = self.model_registry.list_models();
        let mut evaluations = Vec::new();
        let mut eligible_descriptors = Vec::new();

        for model in all_models {
            let mut eligible = true;
            let mut elimination_reason = None;
            let mut detail_msgs = Vec::new();

            // 1. Explicit Model check
            if let Some(explicit) = &req.user_selected_model {
                if !model.id.eq_ignore_ascii_case(explicit)
                    && !model.model_identifier.eq_ignore_ascii_case(explicit)
                {
                    eligible = false;
                    elimination_reason =
                        Some(CandidateEliminationReason::ExplicitSelectionMismatch {
                            selected: explicit.clone(),
                            candidate: model.id.clone(),
                        });
                    detail_msgs.push(format!(
                        "Model does not match explicit selection '{}'",
                        explicit
                    ));
                }
            }

            // 2. Provider Health check
            if eligible {
                let (healthy, health_reason) =
                    self.health_evaluator.evaluate_provider(&model.provider_id);
                if !healthy {
                    eligible = false;
                    elimination_reason = Some(CandidateEliminationReason::ProviderUnavailable {
                        status: health_reason.clone(),
                    });
                    detail_msgs.push(health_reason);
                }
            }

            // 3. Execution Mode filter based on user RoutingMode request
            if eligible {
                let mode_ok = match req.routing_mode {
                    RoutingMode::LocalOnly => model.execution_mode == ExecutionMode::Local,
                    RoutingMode::OnPremiseOnly => model.execution_mode == ExecutionMode::OnPremise,
                    RoutingMode::OnlineOnly => model.execution_mode == ExecutionMode::Cloud,
                    RoutingMode::Auto | RoutingMode::ExplicitModel => true,
                };

                if !mode_ok {
                    eligible = false;
                    elimination_reason = Some(CandidateEliminationReason::ExecutionModeForbidden {
                        mode: model.execution_mode,
                        reason: format!(
                            "Execution mode '{}' violates routing mode preference '{}'",
                            model.execution_mode, req.routing_mode
                        ),
                    });
                    detail_msgs.push(format!(
                        "Execution mode '{}' forbidden by mode setting",
                        model.execution_mode
                    ));
                }
            }

            // 4. Authoritative Privacy & Policy Engine evaluation
            if eligible {
                let (policy_ok, decision_state, source, policy_reason) = self
                    .policy_evaluator
                    .evaluate_execution_mode(req, model.execution_mode, workspace_classification);

                if !policy_ok {
                    eligible = false;
                    elimination_reason = Some(CandidateEliminationReason::PolicyDenied {
                        rule_id: source.to_string(),
                        reason: policy_reason.clone(),
                    });
                    detail_msgs.push(format!(
                        "Policy denied: {} ({:?})",
                        policy_reason, decision_state
                    ));
                }
            }

            // 5. Hardware Compatibility evaluation
            let mut hardware_status = CompatibilityStatus::Compatible;
            if eligible {
                let (hw_ok, hw_res, hw_msg) = self.hardware_evaluator.evaluate_candidate(&model);
                hardware_status = hw_res.status;

                if !hw_ok {
                    eligible = false;
                    elimination_reason = Some(CandidateEliminationReason::HardwareIncompatible {
                        details: hw_msg.clone(),
                    });
                    detail_msgs.push(format!("Hardware incompatible: {}", hw_msg));
                }
            }

            // 6. Context Window evaluation
            if eligible {
                let (ctx_ok, total_req, max_ctx, ctx_msg) = ContextEvaluator::evaluate_candidate(
                    &model,
                    req.required_context_tokens,
                    req.required_output_tokens,
                );

                if !ctx_ok {
                    eligible = false;
                    elimination_reason = Some(CandidateEliminationReason::ContextTooSmall {
                        required: total_req,
                        available: max_ctx,
                    });
                    detail_msgs.push(ctx_msg);
                }
            }

            // 7. Capability matching
            if eligible {
                let inferred = if req.required_capabilities.is_empty() {
                    CapabilityEvaluator::infer_capabilities_for_purpose(&req.purpose)
                } else {
                    req.required_capabilities.clone()
                };

                let (caps_ok, missing_cap, cap_msg) =
                    CapabilityEvaluator::evaluate_capabilities(&model, &inferred);

                if !caps_ok {
                    eligible = false;
                    elimination_reason = Some(CandidateEliminationReason::CapabilityMissing {
                        missing_capability: missing_cap
                            .map(|c| format!("{:?}", c))
                            .unwrap_or_default(),
                    });
                    detail_msgs.push(cap_msg);
                }
            }

            // 8. Scoring
            let score = if eligible {
                ModelScorer::score_candidate(&model, req, hardware_status, workspace_classification)
            } else {
                -1000000.0
            };

            if eligible {
                eligible_descriptors.push(model.clone());
            }

            evaluations.push(CandidateEvaluation {
                model_id: model.id.clone(),
                provider_id: model.provider_id.clone(),
                display_name: model.display_name.clone(),
                execution_mode: model.execution_mode,
                eligible,
                elimination_reason,
                score,
                details: detail_msgs.join("; "),
            });
        }

        // Sort candidates by score descending
        evaluations.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        eligible_descriptors.sort_by(|a, b| {
            let score_a = ModelScorer::score_candidate(
                a,
                req,
                CompatibilityStatus::Compatible,
                workspace_classification,
            );
            let score_b = ModelScorer::score_candidate(
                b,
                req,
                CompatibilityStatus::Compatible,
                workspace_classification,
            );
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        (evaluations, eligible_descriptors)
    }
}
