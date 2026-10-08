use crate::tools::types::{RiskLevel, ToolCapability, ToolExecutionStatus, ToolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolDecisionState {
    Allow,
    Block,
    RequireConsent,
    Unknown,
}

impl std::fmt::Display for ToolDecisionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolDecisionState::Allow => write!(f, "ALLOW"),
            ToolDecisionState::Block => write!(f, "BLOCK"),
            ToolDecisionState::RequireConsent => write!(f, "REQUIRE_CONSENT"),
            ToolDecisionState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDecision {
    pub decision: ToolDecisionState,
    pub reason: String,
    pub matched_policy: Option<String>,
    pub policy_source: String,
    pub granted_capabilities: Vec<ToolCapability>,
    pub denied_capabilities: Vec<ToolCapability>,
    pub warnings: Vec<String>,
    pub risk_level: RiskLevel,
}

impl ToolDecision {
    pub fn is_allowed(&self) -> bool {
        self.decision == ToolDecisionState::Allow
    }

    pub fn is_blocked(&self) -> bool {
        self.decision == ToolDecisionState::Block || self.decision == ToolDecisionState::Unknown
    }

    pub fn requires_consent(&self) -> bool {
        self.decision == ToolDecisionState::RequireConsent
    }

    pub fn block(reason: impl Into<String>, policy_source: impl Into<String>) -> Self {
        Self {
            decision: ToolDecisionState::Block,
            reason: reason.into(),
            matched_policy: None,
            policy_source: policy_source.into(),
            granted_capabilities: vec![],
            denied_capabilities: vec![],
            warnings: vec![],
            risk_level: RiskLevel::Critical,
        }
    }

    pub fn allow(
        reason: impl Into<String>,
        granted: Vec<ToolCapability>,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            decision: ToolDecisionState::Allow,
            reason: reason.into(),
            matched_policy: None,
            policy_source: "tool_policy_engine".to_string(),
            granted_capabilities: granted,
            denied_capabilities: vec![],
            warnings: vec![],
            risk_level,
        }
    }

    pub fn require_consent(
        reason: impl Into<String>,
        requested: Vec<ToolCapability>,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            decision: ToolDecisionState::RequireConsent,
            reason: reason.into(),
            matched_policy: None,
            policy_source: "security_consent_policy".to_string(),
            granted_capabilities: vec![],
            denied_capabilities: requested,
            warnings: vec!["User explicit consent required before tool execution".to_string()],
            risk_level,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub tool_call_id: String,
    pub tool_id: ToolId,
    pub status: ToolExecutionStatus,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub security_notices: Vec<String>,
    pub duration_ms: u128,
}

impl ToolResponse {
    pub fn success(
        tool_call_id: impl Into<String>,
        tool_id: ToolId,
        result: serde_json::Value,
        duration_ms: u128,
    ) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            tool_id,
            status: ToolExecutionStatus::Valid,
            result,
            error: None,
            warnings: vec![],
            security_notices: vec![],
            duration_ms,
        }
    }

    pub fn blocked(
        tool_call_id: impl Into<String>,
        tool_id: ToolId,
        reason: impl Into<String>,
    ) -> Self {
        let msg = reason.into();
        Self {
            tool_call_id: tool_call_id.into(),
            tool_id,
            status: ToolExecutionStatus::Blocked,
            result: serde_json::json!({ "error": msg }),
            error: Some(msg),
            warnings: vec![],
            security_notices: vec!["Execution blocked by Octrex Tool Security Boundary".to_string()],
            duration_ms: 0,
        }
    }

    pub fn failed(
        tool_call_id: impl Into<String>,
        tool_id: ToolId,
        err_msg: impl Into<String>,
        duration_ms: u128,
    ) -> Self {
        let msg = err_msg.into();
        Self {
            tool_call_id: tool_call_id.into(),
            tool_id,
            status: ToolExecutionStatus::Failed,
            result: serde_json::json!({ "error": msg }),
            error: Some(msg),
            warnings: vec![],
            security_notices: vec![],
            duration_ms,
        }
    }
}
