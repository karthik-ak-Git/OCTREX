use crate::privacy::TrustLevel;
use serde::{Deserialize, Serialize};

/// Categories of information sources entering the Context Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    SystemPolicy,
    CompanyPolicy,
    SecurityPolicy,
    PrivacyPolicy,
    UserRequest,
    ConversationMessage,
    TaskState,
    WorkspaceContext,
    FileContent,
    ToolResult,
    ModelOutput,
    VerificationResult,
    CompactionSummary,
    ArtifactReference,
    RuntimeInstruction,
}

impl ContextSource {
    pub fn is_control_plane(&self) -> bool {
        matches!(
            self,
            ContextSource::SystemPolicy
                | ContextSource::CompanyPolicy
                | ContextSource::SecurityPolicy
                | ContextSource::PrivacyPolicy
                | ContextSource::RuntimeInstruction
        )
    }

    pub fn is_data_plane(&self) -> bool {
        !self.is_control_plane()
    }

    pub fn default_trust(&self) -> ContextTrustLevel {
        match self {
            ContextSource::SystemPolicy => ContextTrustLevel::TrustedSystem,
            ContextSource::CompanyPolicy => ContextTrustLevel::TrustedCompany,
            ContextSource::SecurityPolicy => ContextTrustLevel::TrustedSecurity,
            ContextSource::PrivacyPolicy => ContextTrustLevel::TrustedPrivacy,
            ContextSource::UserRequest => ContextTrustLevel::UserControlled,
            ContextSource::ConversationMessage => ContextTrustLevel::UserControlled,
            ContextSource::TaskState => ContextTrustLevel::TrustedSystem,
            ContextSource::WorkspaceContext => ContextTrustLevel::UserControlled,
            ContextSource::FileContent => ContextTrustLevel::UntrustedFile,
            ContextSource::ToolResult => ContextTrustLevel::UntrustedTool,
            ContextSource::ModelOutput => ContextTrustLevel::ModelGenerated,
            ContextSource::VerificationResult => ContextTrustLevel::TrustedSystem,
            ContextSource::CompactionSummary => ContextTrustLevel::ModelGenerated,
            ContextSource::ArtifactReference => ContextTrustLevel::UserControlled,
            ContextSource::RuntimeInstruction => ContextTrustLevel::TrustedSystem,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContextSource::SystemPolicy => "system_policy",
            ContextSource::CompanyPolicy => "company_policy",
            ContextSource::SecurityPolicy => "security_policy",
            ContextSource::PrivacyPolicy => "privacy_policy",
            ContextSource::UserRequest => "user_request",
            ContextSource::ConversationMessage => "conversation_message",
            ContextSource::TaskState => "task_state",
            ContextSource::WorkspaceContext => "workspace_context",
            ContextSource::FileContent => "file_content",
            ContextSource::ToolResult => "tool_result",
            ContextSource::ModelOutput => "model_output",
            ContextSource::VerificationResult => "verification_result",
            ContextSource::CompactionSummary => "compaction_summary",
            ContextSource::ArtifactReference => "artifact_reference",
            ContextSource::RuntimeInstruction => "runtime_instruction",
        }
    }
}

/// Normalized roles for prompt assembly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRole {
    System,
    User,
    Assistant,
    Tool,
    FileContent,
    ToolResult,
    ModelOutput,
    CompactionSummary,
}

impl ContextRole {
    pub fn is_privileged(&self) -> bool {
        matches!(self, ContextRole::System)
    }

    pub fn to_model_role(&self) -> &'static str {
        match self {
            ContextRole::System => "system",
            ContextRole::User | ContextRole::FileContent => "user",
            ContextRole::Assistant | ContextRole::ModelOutput | ContextRole::CompactionSummary => {
                "assistant"
            }
            ContextRole::Tool | ContextRole::ToolResult => "tool",
        }
    }
}

/// Granular security trust levels for context items
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextTrustLevel {
    ExternalData = 0,
    UntrustedWeb = 10,
    UntrustedTool = 20,
    UntrustedFile = 30,
    ModelGenerated = 40,
    UserControlled = 50,
    TrustedPrivacy = 60,
    TrustedSecurity = 70,
    TrustedCompany = 80,
    TrustedSystem = 100,
}

impl ContextTrustLevel {
    pub fn is_untrusted(&self) -> bool {
        *self <= ContextTrustLevel::UntrustedFile
    }

    pub fn to_privacy_trust(&self) -> TrustLevel {
        match self {
            ContextTrustLevel::TrustedSystem
            | ContextTrustLevel::TrustedCompany
            | ContextTrustLevel::TrustedSecurity
            | ContextTrustLevel::TrustedPrivacy => TrustLevel::TrustedSystem,
            ContextTrustLevel::UserControlled => TrustLevel::TrustedUser,
            ContextTrustLevel::ModelGenerated
            | ContextTrustLevel::UntrustedFile
            | ContextTrustLevel::UntrustedTool => TrustLevel::UntrustedInput,
            ContextTrustLevel::UntrustedWeb | ContextTrustLevel::ExternalData => {
                TrustLevel::UntrustedExternal
            }
        }
    }
}

/// Reason for including a context item in prompt assembly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInclusionReason {
    Mandatory,
    CurrentTask,
    Recent,
    Relevant,
    Dependency,
    VerificationRequired,
    UserSelected,
    PolicyRequired,
}

/// Token counting precision category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenCountKind {
    Exact,
    Estimated,
    Unknown,
}

impl Default for TokenCountKind {
    fn default() -> Self {
        TokenCountKind::Unknown
    }
}
