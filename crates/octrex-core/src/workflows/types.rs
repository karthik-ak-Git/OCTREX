use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::privacy::PrivacyClassification;
use crate::skills::types::SkillSource;
use crate::tools::ToolCapability;

/// Workflow step kinds. Execution always passes through the authoritative
/// runtime boundary (ToolRuntime / Filesystem / ModelRuntime / Verification).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowStepKind {
    Skill,
    Tool,
    ModelOperation,
    FileOperation,
    Verification,
    UserInput,
}

impl fmt::Display for WorkflowStepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowStepKind::Skill => write!(f, "SKILL"),
            WorkflowStepKind::Tool => write!(f, "TOOL"),
            WorkflowStepKind::ModelOperation => write!(f, "MODEL_OPERATION"),
            WorkflowStepKind::FileOperation => write!(f, "FILE_OPERATION"),
            WorkflowStepKind::Verification => write!(f, "VERIFICATION"),
            WorkflowStepKind::UserInput => write!(f, "USER_INPUT"),
        }
    }
}

impl FromStr for WorkflowStepKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "SKILL" => Ok(WorkflowStepKind::Skill),
            "TOOL" => Ok(WorkflowStepKind::Tool),
            "MODEL_OPERATION" | "MODELOPERATION" | "MODEL" => Ok(WorkflowStepKind::ModelOperation),
            "FILE_OPERATION" | "FILEOPERATION" | "FILE" => Ok(WorkflowStepKind::FileOperation),
            "VERIFICATION" | "VERIFY" => Ok(WorkflowStepKind::Verification),
            "USER_INPUT" | "USERINPUT" | "USER" => Ok(WorkflowStepKind::UserInput),
            other => Err(format!("Unknown workflow step kind: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub kind: WorkflowStepKind,
    /// Skill id / tool id / file reference / verification check name.
    pub reference: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub required_capabilities: Vec<ToolCapability>,
    pub inputs: Option<serde_json::Value>,
    pub expected_output: Option<String>,
    pub verification_required: bool,
}

impl WorkflowStep {
    pub fn new(
        id: impl Into<String>,
        kind: WorkflowStepKind,
        reference: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            reference: reference.into(),
            description: description.into(),
            dependencies: Vec::new(),
            required_capabilities: Vec::new(),
            inputs: None,
            expected_output: None,
            verification_required: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowStatus {
    #[default]
    Draft,
    Active,
    Disabled,
    Blocked,
}

impl fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowStatus::Draft => write!(f, "DRAFT"),
            WorkflowStatus::Active => write!(f, "ACTIVE"),
            WorkflowStatus::Disabled => write!(f, "DISABLED"),
            WorkflowStatus::Blocked => write!(f, "BLOCKED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: SkillSource,
    pub status: WorkflowStatus,
    pub classification: PrivacyClassification,
    pub inputs: serde_json::Value,
    pub steps: Vec<WorkflowStep>,
    pub outputs: serde_json::Value,
    pub verification: Vec<String>,
    pub policy_requirements: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl WorkflowDefinition {
    pub fn versioned_id(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for WorkflowRunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowRunStatus::Pending => write!(f, "PENDING"),
            WorkflowRunStatus::Running => write!(f, "RUNNING"),
            WorkflowRunStatus::Completed => write!(f, "COMPLETED"),
            WorkflowRunStatus::Failed => write!(f, "FAILED"),
            WorkflowRunStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: String,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub workspace_id: Option<String>,
    pub status: WorkflowRunStatus,
    pub inputs: serde_json::Value,
    pub outputs: Option<serde_json::Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
