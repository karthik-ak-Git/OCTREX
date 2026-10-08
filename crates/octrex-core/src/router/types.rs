use crate::hardware::CompatibilityResult;
use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use crate::models::types::ModelDescriptor;
use crate::privacy::{PrivacyClassification, PrivacyMode};
use crate::providers::ExecutionMode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// User-selectable or System Routing Modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    Auto,
    LocalOnly,
    OnPremiseOnly,
    OnlineOnly,
    ExplicitModel,
}

impl Default for RoutingMode {
    fn default() -> Self {
        RoutingMode::Auto
    }
}

impl fmt::Display for RoutingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoutingMode::Auto => write!(f, "auto"),
            RoutingMode::LocalOnly => write!(f, "local_only"),
            RoutingMode::OnPremiseOnly => write!(f, "on_premise_only"),
            RoutingMode::OnlineOnly => write!(f, "online_only"),
            RoutingMode::ExplicitModel => write!(f, "explicit_model"),
        }
    }
}

impl FromStr for RoutingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "auto" => Ok(RoutingMode::Auto),
            "local_only" | "local" => Ok(RoutingMode::LocalOnly),
            "on_premise_only" | "on_premise" | "onprem" => Ok(RoutingMode::OnPremiseOnly),
            "online_only" | "online" | "cloud" => Ok(RoutingMode::OnlineOnly),
            "explicit_model" | "explicit" => Ok(RoutingMode::ExplicitModel),
            other => Err(format!("Unknown routing mode: {}", other)),
        }
    }
}

impl RoutingMode {
    pub fn from_privacy_mode(pm: PrivacyMode) -> Self {
        match pm {
            PrivacyMode::Auto => RoutingMode::Auto,
            PrivacyMode::LocalOnly => RoutingMode::LocalOnly,
            PrivacyMode::OnlineOnly => RoutingMode::OnlineOnly,
            PrivacyMode::Confidential => RoutingMode::LocalOnly,
        }
    }
}

/// Final outcome state of a Routing Decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDecisionState {
    Selected,
    RequireUserSelection,
    Blocked,
    NoCompatibleModel,
    NoAuthorizedModel,
    ContextTooLarge,
    HardwareIncompatible,
    ProviderUnavailable,
    PolicyDenied,
    Unknown,
}

impl fmt::Display for RoutingDecisionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoutingDecisionState::Selected => write!(f, "selected"),
            RoutingDecisionState::RequireUserSelection => write!(f, "require_user_selection"),
            RoutingDecisionState::Blocked => write!(f, "blocked"),
            RoutingDecisionState::NoCompatibleModel => write!(f, "no_compatible_model"),
            RoutingDecisionState::NoAuthorizedModel => write!(f, "no_authorized_model"),
            RoutingDecisionState::ContextTooLarge => write!(f, "context_too_large"),
            RoutingDecisionState::HardwareIncompatible => write!(f, "hardware_incompatible"),
            RoutingDecisionState::ProviderUnavailable => write!(f, "provider_unavailable"),
            RoutingDecisionState::PolicyDenied => write!(f, "policy_denied"),
            RoutingDecisionState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Structured reasons for candidate model elimination during candidate pipeline filtering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason_type", content = "details")]
pub enum CandidateEliminationReason {
    ExecutionModeForbidden {
        mode: ExecutionMode,
        reason: String,
    },
    PolicyDenied {
        rule_id: String,
        reason: String,
    },
    PrivacyDenied {
        classification: PrivacyClassification,
        reason: String,
    },
    HardwareIncompatible {
        details: String,
    },
    ContextTooSmall {
        required: u32,
        available: u32,
    },
    CapabilityMissing {
        missing_capability: String,
    },
    ProviderUnavailable {
        status: String,
    },
    Disabled,
    ExplicitSelectionMismatch {
        selected: String,
        candidate: String,
    },
}

/// Detail evaluation result for a candidate model in candidate pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateEvaluation {
    pub model_id: String,
    pub provider_id: String,
    pub display_name: String,
    pub execution_mode: ExecutionMode,
    pub eligible: bool,
    pub elimination_reason: Option<CandidateEliminationReason>,
    pub score: f64,
    pub details: String,
}

/// Structured evidence collected during the routing pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingEvidence {
    pub candidate_count: usize,
    pub eligible_count: usize,
    pub excluded_candidates: Vec<CandidateEvaluation>,
    pub selected_candidate: Option<CandidateEvaluation>,
    pub privacy_status: String,
    pub hardware_status: String,
    pub context_status: String,
    pub provider_status: String,
    pub policy_status: String,
}

/// Canonical Routing Decision returned by ModelRouter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub id: String,
    pub request_id: RequestId,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub selected_model_id: Option<String>,
    pub selected_provider_id: Option<String>,
    pub selected_descriptor: Option<ModelDescriptor>,
    pub execution_mode: Option<ExecutionMode>,
    pub state: RoutingDecisionState,
    pub reason: String,
    pub confidence: f64,
    pub timestamp: u64,
    pub evidence: RoutingEvidence,
    pub policy_evidence: Vec<String>,
    pub hardware_compatibility: Option<CompatibilityResult>,
    pub context_compatibility: Option<String>,
}

impl RoutingDecision {
    pub fn is_selected(&self) -> bool {
        self.state == RoutingDecisionState::Selected && self.selected_model_id.is_some()
    }
}
