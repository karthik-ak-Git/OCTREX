use crate::ids::{ArtifactId, SessionId, TaskId, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "USER"),
            MessageRole::Assistant => write!(f, "ASSISTANT"),
            MessageRole::System => write!(f, "SYSTEM"),
            MessageRole::Tool => write!(f, "TOOL"),
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USER" => Ok(MessageRole::User),
            "ASSISTANT" => Ok(MessageRole::Assistant),
            "SYSTEM" => Ok(MessageRole::System),
            "TOOL" => Ok(MessageRole::Tool),
            other => Err(format!("Unknown message role: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: SessionId,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for TaskStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStepStatus::Pending => write!(f, "PENDING"),
            TaskStepStatus::Running => write!(f, "RUNNING"),
            TaskStepStatus::Completed => write!(f, "COMPLETED"),
            TaskStepStatus::Failed => write!(f, "FAILED"),
        }
    }
}

impl std::str::FromStr for TaskStepStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PENDING" => Ok(TaskStepStatus::Pending),
            "RUNNING" => Ok(TaskStepStatus::Running),
            "COMPLETED" => Ok(TaskStepStatus::Completed),
            "FAILED" => Ok(TaskStepStatus::Failed),
            other => Err(format!("Unknown task step status: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub task_id: TaskId,
    pub sequence: u32,
    pub objective: String,
    pub status: TaskStepStatus,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub id: String,
    pub provider_id: String,
    pub model_identifier: String,
    pub display_name: String,
    pub enabled: bool,
    pub local_or_remote: String,
    pub context_window: u32,
    pub capabilities: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRecord {
    pub category: String,
    pub key: String,
    pub value: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}

impl std::fmt::Display for PermissionDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionDecision::Allow => write!(f, "ALLOW"),
            PermissionDecision::Deny => write!(f, "DENY"),
            PermissionDecision::Ask => write!(f, "ASK"),
        }
    }
}

impl std::str::FromStr for PermissionDecision {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ALLOW" => Ok(PermissionDecision::Allow),
            "DENY" => Ok(PermissionDecision::Deny),
            "ASK" => Ok(PermissionDecision::Ask),
            other => Err(format!("Unknown permission decision: {}", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionDuration {
    Once,
    Task,
    Session,
    Persistent,
}

impl std::fmt::Display for PermissionDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionDuration::Once => write!(f, "ONCE"),
            PermissionDuration::Task => write!(f, "TASK"),
            PermissionDuration::Session => write!(f, "SESSION"),
            PermissionDuration::Persistent => write!(f, "PERSISTENT"),
        }
    }
}

impl std::str::FromStr for PermissionDuration {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ONCE" => Ok(PermissionDuration::Once),
            "TASK" => Ok(PermissionDuration::Task),
            "SESSION" => Ok(PermissionDuration::Session),
            "PERSISTENT" => Ok(PermissionDuration::Persistent),
            other => Err(format!("Unknown permission duration: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRecord {
    pub id: String,
    pub scope: String,
    pub resource: String,
    pub decision: PermissionDecision,
    pub duration: PermissionDuration,
    pub workspace_id: Option<WorkspaceId>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: ArtifactId,
    pub task_id: Option<TaskId>,
    pub workspace_id: Option<WorkspaceId>,
    pub name: String,
    pub path: String,
    pub artifact_type: String,
    pub size: u64,
    pub checksum: Option<String>,
    pub created_at: u64,
    pub verification_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: u64,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub workspace_id: Option<WorkspaceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub timestamp: u64,
    pub event_type: String,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub actor: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub route: Option<String>,
    pub privacy_classification: Option<String>,
    pub policy_source: Option<String>,
    pub permission: Option<String>,
    pub tool: Option<String>,
    pub success: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: String,
    pub enabled: bool,
    pub trust_level: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnectionRecord {
    pub id: String,
    pub name: String,
    pub server_type: String,
    pub configuration: serde_json::Value,
    pub enabled: bool,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
}
