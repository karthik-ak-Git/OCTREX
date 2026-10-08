use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use crate::providers::ExecutionMode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Strongly typed Privacy Data Classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrivacyClassification {
    Public = 1,
    Internal = 2,
    Confidential = 3,
    Restricted = 4,
    Secret = 5,
}

impl Default for PrivacyClassification {
    fn default() -> Self {
        PrivacyClassification::Public
    }
}

impl fmt::Display for PrivacyClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrivacyClassification::Public => write!(f, "PUBLIC"),
            PrivacyClassification::Internal => write!(f, "INTERNAL"),
            PrivacyClassification::Confidential => write!(f, "CONFIDENTIAL"),
            PrivacyClassification::Restricted => write!(f, "RESTRICTED"),
            PrivacyClassification::Secret => write!(f, "SECRET"),
        }
    }
}

impl FromStr for PrivacyClassification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "PUBLIC" => Ok(PrivacyClassification::Public),
            "INTERNAL" => Ok(PrivacyClassification::Internal),
            "CONFIDENTIAL" => Ok(PrivacyClassification::Confidential),
            "RESTRICTED" => Ok(PrivacyClassification::Restricted),
            "SECRET" => Ok(PrivacyClassification::Secret),
            other => Err(format!("Unknown classification: {}", other)),
        }
    }
}

/// User-selectable Privacy Modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrivacyMode {
    Auto,
    LocalOnly,
    OnlineOnly,
    Confidential,
}

impl Default for PrivacyMode {
    fn default() -> Self {
        PrivacyMode::LocalOnly
    }
}

impl fmt::Display for PrivacyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrivacyMode::Auto => write!(f, "AUTO"),
            PrivacyMode::LocalOnly => write!(f, "LOCAL_ONLY"),
            PrivacyMode::OnlineOnly => write!(f, "ONLINE_ONLY"),
            PrivacyMode::Confidential => write!(f, "CONFIDENTIAL"),
        }
    }
}

impl FromStr for PrivacyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "AUTO" => Ok(PrivacyMode::Auto),
            "LOCAL_ONLY" | "LOCALONLY" | "STRICT_LOCAL" => Ok(PrivacyMode::LocalOnly),
            "ONLINE_ONLY" | "ONLINEONLY" => Ok(PrivacyMode::OnlineOnly),
            "CONFIDENTIAL" => Ok(PrivacyMode::Confidential),
            other => Err(format!("Unknown privacy mode: {}", other)),
        }
    }
}

/// Explicit Policy Hierarchy Hierarchy Priority: System(100) > Company(80) > Security(60) > Privacy(40) > User(20)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicySource {
    User = 20,
    Privacy = 40,
    Security = 60,
    Company = 80,
    System = 100,
}

impl Default for PolicySource {
    fn default() -> Self {
        PolicySource::System
    }
}

impl fmt::Display for PolicySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicySource::User => write!(f, "USER"),
            PolicySource::Privacy => write!(f, "PRIVACY"),
            PolicySource::Security => write!(f, "SECURITY"),
            PolicySource::Company => write!(f, "COMPANY"),
            PolicySource::System => write!(f, "SYSTEM"),
        }
    }
}

impl FromStr for PolicySource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "USER" => Ok(PolicySource::User),
            "PRIVACY" => Ok(PolicySource::Privacy),
            "SECURITY" => Ok(PolicySource::Security),
            "COMPANY" => Ok(PolicySource::Company),
            "SYSTEM" => Ok(PolicySource::System),
            other => Err(format!("Unknown policy source: {}", other)),
        }
    }
}

/// Action specified by policy rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireConsent,
    RequireReview,
}

/// Strongly typed Privacy Decision Outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionState {
    AllowLocal,
    AllowOnPremise,
    AllowOnline,
    DenyOnline,
    Deny,
    RequireConsent,
    RequireReview,
}

impl fmt::Display for DecisionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionState::AllowLocal => write!(f, "ALLOW_LOCAL"),
            DecisionState::AllowOnPremise => write!(f, "ALLOW_ON_PREMISE"),
            DecisionState::AllowOnline => write!(f, "ALLOW_ONLINE"),
            DecisionState::DenyOnline => write!(f, "DENY_ONLINE"),
            DecisionState::Deny => write!(f, "DENY"),
            DecisionState::RequireConsent => write!(f, "REQUIRE_CONSENT"),
            DecisionState::RequireReview => write!(f, "REQUIRE_REVIEW"),
        }
    }
}

/// Classification input content source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputSourceType {
    UserPrompt,
    SelectedFile,
    Workspace,
    FileContent,
    ToolResult,
    TerminalOutput,
    ModelContext,
    PastedText,
    GeneratedArtifact,
    McpResult,
    WebResult,
}

/// Security Trust Level of inputs (Untrusted content can NEVER modify policy!)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrustLevel {
    TrustedSystem,
    TrustedUser,
    UntrustedInput,
    UntrustedExternal,
}

/// Abstraction for content subject to data classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyInput {
    pub source_type: InputSourceType,
    pub content: String,
    pub origin: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub trust_level: TrustLevel,
}

impl PrivacyInput {
    pub fn new(source_type: InputSourceType, content: impl Into<String>) -> Self {
        Self {
            source_type,
            content: content.into(),
            origin: None,
            metadata: None,
            trust_level: TrustLevel::UntrustedInput,
        }
    }

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    pub fn with_trust_level(mut self, trust_level: TrustLevel) -> Self {
        self.trust_level = trust_level;
        self
    }
}

/// Confidence rating for data classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClassificationConfidence {
    Low,
    Medium,
    High,
}

/// Signals emitted during data classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSignal {
    pub category: String,
    pub signal_type: String,
    pub summary: String,
    pub severity: String,
}

/// Result from data classification engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub classification: PrivacyClassification,
    pub confidence: ClassificationConfidence,
    pub signals: Vec<ClassificationSignal>,
    pub evidence_summary: String,
}

/// Target scope of a policy rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyScope {
    Global,
    Workspace(String),
    Classification(PrivacyClassification),
}

/// Typed Policy Rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub source: PolicySource,
    pub priority: u32,
    pub enabled: bool,
    pub scope: PolicyScope,
    pub target_classification: Option<PrivacyClassification>,
    pub target_mode: Option<ExecutionMode>,
    pub forbidden_providers: Vec<String>,
    pub forbidden_models: Vec<String>,
    pub action: PolicyAction,
    pub reason: String,
    pub version: u32,
}

/// Context passed to Privacy Gate & Policy Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyContext {
    pub request_id: RequestId,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub workspace_id: Option<WorkspaceId>,
    pub workspace_classification: PrivacyClassification,
    pub user_privacy_mode: PrivacyMode,
    pub confidential_mode_enabled: bool,
    pub inputs: Vec<PrivacyInput>,
    pub requested_mode: ExecutionMode,
    pub candidate_provider: Option<String>,
    pub candidate_model: Option<String>,
    pub consent_granted: bool,
}

impl PrivacyContext {
    pub fn new(request_id: RequestId) -> Self {
        Self {
            request_id,
            session_id: None,
            task_id: None,
            workspace_id: None,
            workspace_classification: PrivacyClassification::Public,
            user_privacy_mode: PrivacyMode::LocalOnly,
            confidential_mode_enabled: false,
            inputs: Vec::new(),
            requested_mode: ExecutionMode::Local,
            candidate_provider: None,
            candidate_model: None,
            consent_granted: false,
        }
    }
}

/// Canonical Privacy Decision output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyDecision {
    pub id: String,
    pub request_id: RequestId,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub workspace_id: Option<WorkspaceId>,
    pub classification: PrivacyClassification,
    pub requested_execution_mode: ExecutionMode,
    pub allowed_execution_modes: Vec<ExecutionMode>,
    pub selected_policy_source: PolicySource,
    pub policy_version: u32,
    pub decision: DecisionState,
    pub reason: String,
    pub confidence: ClassificationConfidence,
    pub timestamp: u64,
    pub evidence_summary: String,
}

/// Safe Outbound Payload Preview for online execution consent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundPayloadPreview {
    pub request_id: RequestId,
    pub target_provider: String,
    pub target_model: String,
    pub execution_mode: ExecutionMode,
    pub classification: PrivacyClassification,
    pub data_categories: Vec<String>,
    pub files_included: Vec<String>,
    pub approximate_payload_bytes: usize,
    pub sensitive_fields_detected: Vec<String>,
    pub policy_decision: DecisionState,
}

/// User Consent Request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequest {
    pub id: String,
    pub request_id: RequestId,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub classification: PrivacyClassification,
    pub destination_provider: String,
    pub destination_model: String,
    pub requested_mode: ExecutionMode,
    pub payload_preview: OutboundPayloadPreview,
    pub reasoning: String,
    pub created_at: u64,
}

/// User Consent Decision DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentDecision {
    pub id: String,
    pub request_id: RequestId,
    pub consent_id: String,
    pub granted: bool,
    pub reason: Option<String>,
    pub timestamp: u64,
}

/// Overall System Effective Privacy Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivePrivacyStatus {
    pub privacy_mode: PrivacyMode,
    pub confidential_mode: bool,
    pub active_workspace_id: Option<String>,
    pub active_workspace_classification: PrivacyClassification,
    pub active_policies_count: usize,
    pub system_policy_status: String,
    pub company_policy_status: String,
    pub default_mode: PrivacyMode,
}
