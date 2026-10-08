use crate::ids::WorkspaceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub String);

impl ToolId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ToolId {
    fn from(s: &str) -> Self {
        ToolId(s.to_string())
    }
}

impl From<String> for ToolId {
    fn from(s: String) -> Self {
        ToolId(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolSource {
    BuiltIn,
    Skill,
    MCP,
    Provider,
    System,
    UserInstalled,
}

impl fmt::Display for ToolSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolSource::BuiltIn => write!(f, "BUILT_IN"),
            ToolSource::Skill => write!(f, "SKILL"),
            ToolSource::MCP => write!(f, "MCP"),
            ToolSource::Provider => write!(f, "PROVIDER"),
            ToolSource::System => write!(f, "SYSTEM"),
            ToolSource::UserInstalled => write!(f, "USER_INSTALLED"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolCapability {
    // Filesystem Capabilities
    FilesystemRead,
    FilesystemWrite,
    FilesystemCreate,
    FilesystemDelete,
    FilesystemList,
    FilesystemExport,

    // Network Capabilities
    ExternalHttps,
    ExternalHttp,
    ProviderApi,
    WebFetch,
    WebSearch,
    RemoteMcp,

    // Process Capabilities
    ProcessExecute,
    ProcessSpawn,

    // Workspace Capabilities
    WorkspaceRead,
    WorkspaceWrite,
    WorkspaceExport,

    // Data Privacy Capabilities
    ReadConfidentialData,
    ReadRestrictedData,
    ReadSecretData,

    // System Capabilities
    SystemInfo,
    EnvironmentAccess,

    // Custom or dynamic capability fallback (must be explicitly checked)
    Custom(String),
}

impl fmt::Display for ToolCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolCapability::FilesystemRead => write!(f, "FILESYSTEM_READ"),
            ToolCapability::FilesystemWrite => write!(f, "FILESYSTEM_WRITE"),
            ToolCapability::FilesystemCreate => write!(f, "FILESYSTEM_CREATE"),
            ToolCapability::FilesystemDelete => write!(f, "FILESYSTEM_DELETE"),
            ToolCapability::FilesystemList => write!(f, "FILESYSTEM_LIST"),
            ToolCapability::FilesystemExport => write!(f, "FILESYSTEM_EXPORT"),
            ToolCapability::ExternalHttps => write!(f, "EXTERNAL_HTTPS"),
            ToolCapability::ExternalHttp => write!(f, "EXTERNAL_HTTP"),
            ToolCapability::ProviderApi => write!(f, "PROVIDER_API"),
            ToolCapability::WebFetch => write!(f, "WEB_FETCH"),
            ToolCapability::WebSearch => write!(f, "WEB_SEARCH"),
            ToolCapability::RemoteMcp => write!(f, "REMOTE_MCP"),
            ToolCapability::ProcessExecute => write!(f, "PROCESS_EXECUTE"),
            ToolCapability::ProcessSpawn => write!(f, "PROCESS_SPAWN"),
            ToolCapability::WorkspaceRead => write!(f, "WORKSPACE_READ"),
            ToolCapability::WorkspaceWrite => write!(f, "WORKSPACE_WRITE"),
            ToolCapability::WorkspaceExport => write!(f, "WORKSPACE_EXPORT"),
            ToolCapability::ReadConfidentialData => write!(f, "READ_CONFIDENTIAL_DATA"),
            ToolCapability::ReadRestrictedData => write!(f, "READ_RESTRICTED_DATA"),
            ToolCapability::ReadSecretData => write!(f, "READ_SECRET_DATA"),
            ToolCapability::SystemInfo => write!(f, "SYSTEM_INFO"),
            ToolCapability::EnvironmentAccess => write!(f, "ENVIRONMENT_ACCESS"),
            ToolCapability::Custom(c) => write!(f, "CUSTOM:{}", c),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityScope {
    pub workspace_id: Option<WorkspaceId>,
    pub allowed_paths: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub allowed_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub id: ToolId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: ToolSource,
    pub capabilities: Vec<ToolCapability>,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub risk_level: RiskLevel,
    pub enabled: bool,
    pub requires_confirmation: bool,
    pub timeout_ms: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolExecutionStatus {
    Valid,
    Redacted,
    Truncated,
    Invalid,
    Failed,
    Blocked,
    Cancelled,
    TimedOut,
}
