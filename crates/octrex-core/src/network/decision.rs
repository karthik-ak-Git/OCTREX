use super::endpoint::NetworkEndpoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Allow,
    Block,
    RequireConsent,
    Unknown,
}

impl Disposition {
    /// Invariant enforcement: Unknown resolves to Block.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Block | Self::Unknown)
    }

    pub fn requires_consent(&self) -> bool {
        matches!(self, Self::RequireConsent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicySource {
    System = 1,
    Company = 2,
    Security = 3,
    Privacy = 4,
    Permission = 5,
    User = 6,
}

impl std::fmt::Display for PolicySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system_policy"),
            Self::Company => write!(f, "company_policy"),
            Self::Security => write!(f, "security_policy"),
            Self::Privacy => write!(f, "privacy_policy"),
            Self::Permission => write!(f, "permission_policy"),
            Self::User => write!(f, "user_policy"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDecision {
    pub decision_id: String,
    pub timestamp: u64,
    pub disposition: Disposition,
    pub reason: String,
    pub matched_rule: Option<String>,
    pub policy_source: PolicySource,
    pub endpoint: NetworkEndpoint,
    pub requires_consent: bool,
    pub warnings: Vec<String>,
}

impl NetworkDecision {
    pub fn allow(
        reason: impl Into<String>,
        matched_rule: Option<String>,
        policy_source: PolicySource,
        endpoint: NetworkEndpoint,
    ) -> Self {
        Self {
            decision_id: format!("dec-{}", uuid::Uuid::new_v4()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            disposition: Disposition::Allow,
            reason: reason.into(),
            matched_rule,
            policy_source,
            endpoint,
            requires_consent: false,
            warnings: Vec::new(),
        }
    }

    pub fn block(
        reason: impl Into<String>,
        matched_rule: Option<String>,
        policy_source: PolicySource,
        endpoint: NetworkEndpoint,
    ) -> Self {
        Self {
            decision_id: format!("dec-{}", uuid::Uuid::new_v4()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            disposition: Disposition::Block,
            reason: reason.into(),
            matched_rule,
            policy_source,
            endpoint,
            requires_consent: false,
            warnings: Vec::new(),
        }
    }

    pub fn require_consent(
        reason: impl Into<String>,
        matched_rule: Option<String>,
        policy_source: PolicySource,
        endpoint: NetworkEndpoint,
    ) -> Self {
        Self {
            decision_id: format!("dec-{}", uuid::Uuid::new_v4()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            disposition: Disposition::RequireConsent,
            reason: reason.into(),
            matched_rule,
            policy_source,
            endpoint,
            requires_consent: true,
            warnings: Vec::new(),
        }
    }

    pub fn unknown(
        reason: impl Into<String>,
        policy_source: PolicySource,
        endpoint: NetworkEndpoint,
    ) -> Self {
        Self {
            decision_id: format!("dec-{}", uuid::Uuid::new_v4()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            disposition: Disposition::Unknown, // Enforcement layer converts Unknown -> Block
            reason: reason.into(),
            matched_rule: None,
            policy_source,
            endpoint,
            requires_consent: false,
            warnings: vec![
                "Unknown authorization state - Fail-Closed enforcement applied".to_string(),
            ],
        }
    }

    /// Evaluates if request is permitted. Returns true ONLY if disposition is Allow.
    pub fn is_allowed(&self) -> bool {
        self.disposition.is_allowed()
    }
}
