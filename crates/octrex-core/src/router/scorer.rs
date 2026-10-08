use crate::hardware::CompatibilityStatus;
use crate::models::types::ModelDescriptor;
use crate::privacy::PrivacyClassification;
use crate::providers::ExecutionMode;
use crate::router::request::RoutingRequest;
use crate::router::types::RoutingMode;

pub struct ModelScorer;

impl ModelScorer {
    pub fn score_candidate(
        model: &ModelDescriptor,
        req: &RoutingRequest,
        hardware_status: CompatibilityStatus,
        workspace_classification: PrivacyClassification,
    ) -> f64 {
        let mut score: f64 = 100.0;

        // 1. Workspace Privacy Alignment
        match workspace_classification {
            PrivacyClassification::Confidential
            | PrivacyClassification::Restricted
            | PrivacyClassification::Secret => {
                if model.execution_mode == ExecutionMode::Local {
                    score += 35.0;
                } else if model.execution_mode == ExecutionMode::OnPremise {
                    score += 20.0;
                }
            }
            PrivacyClassification::Internal => {
                if model.execution_mode == ExecutionMode::Local {
                    score += 20.0;
                } else if model.execution_mode == ExecutionMode::OnPremise {
                    score += 15.0;
                }
            }
            PrivacyClassification::Public => {}
        }

        // 2. Execution Mode Preference
        match req.routing_mode {
            RoutingMode::LocalOnly => {
                if model.execution_mode == ExecutionMode::Local {
                    score += 30.0;
                }
            }
            RoutingMode::OnPremiseOnly => {
                if model.execution_mode == ExecutionMode::OnPremise {
                    score += 30.0;
                }
            }
            RoutingMode::OnlineOnly => {
                if model.execution_mode == ExecutionMode::Cloud {
                    score += 30.0;
                }
            }
            RoutingMode::ExplicitModel => {
                if let Some(explicit) = &req.user_selected_model {
                    if model.id.eq_ignore_ascii_case(explicit)
                        || model.model_identifier.eq_ignore_ascii_case(explicit)
                    {
                        score += 100.0;
                    }
                }
            }
            RoutingMode::Auto => {
                // Default privacy preference for local execution in Auto mode
                if model.execution_mode == ExecutionMode::Local {
                    score += 15.0;
                }
            }
        }

        // 3. Hardware Compatibility status score
        match hardware_status {
            CompatibilityStatus::Compatible => score += 20.0,
            CompatibilityStatus::CompatibleWithWarnings => score += 10.0,
            CompatibilityStatus::Incompatible | CompatibilityStatus::Unknown => score -= 1000.0,
        }

        // 4. Context Headroom bonus
        if let Some(cw) = model.context_window {
            let req_tokens = req.required_context_tokens.unwrap_or(2048);
            if cw >= req_tokens {
                let headroom_ratio = (cw - req_tokens) as f64 / cw as f64;
                score += headroom_ratio * 15.0;
            }
        }

        score
    }
}
