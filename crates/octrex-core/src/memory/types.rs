use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::privacy::PrivacyClassification;

/// Memory entry kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryType {
    UserPreference,
    ProjectFact,
    WorkspaceFact,
    TaskFact,
    WorkflowFact,
    SkillFact,
    ArtifactFact,
    Decision,
    Constraint,
    Procedure,
    TemporaryFact,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::UserPreference => write!(f, "USER_PREFERENCE"),
            MemoryType::ProjectFact => write!(f, "PROJECT_FACT"),
            MemoryType::WorkspaceFact => write!(f, "WORKSPACE_FACT"),
            MemoryType::TaskFact => write!(f, "TASK_FACT"),
            MemoryType::WorkflowFact => write!(f, "WORKFLOW_FACT"),
            MemoryType::SkillFact => write!(f, "SKILL_FACT"),
            MemoryType::ArtifactFact => write!(f, "ARTIFACT_FACT"),
            MemoryType::Decision => write!(f, "DECISION"),
            MemoryType::Constraint => write!(f, "CONSTRAINT"),
            MemoryType::Procedure => write!(f, "PROCEDURE"),
            MemoryType::TemporaryFact => write!(f, "TEMPORARY_FACT"),
        }
    }
}

impl FromStr for MemoryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "USER_PREFERENCE" | "USERPREFERENCE" => Ok(MemoryType::UserPreference),
            "PROJECT_FACT" => Ok(MemoryType::ProjectFact),
            "WORKSPACE_FACT" => Ok(MemoryType::WorkspaceFact),
            "TASK_FACT" => Ok(MemoryType::TaskFact),
            "WORKFLOW_FACT" => Ok(MemoryType::WorkflowFact),
            "SKILL_FACT" => Ok(MemoryType::SkillFact),
            "ARTIFACT_FACT" => Ok(MemoryType::ArtifactFact),
            "DECISION" => Ok(MemoryType::Decision),
            "CONSTRAINT" => Ok(MemoryType::Constraint),
            "PROCEDURE" => Ok(MemoryType::Procedure),
            "TEMPORARY_FACT" => Ok(MemoryType::TemporaryFact),
            other => Err(format!("Unknown memory type: {}", other)),
        }
    }
}

/// Strict memory scopes. Never auto-expose across workspaces/tasks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryScope {
    Global,
    Workspace,
    Project,
    Session,
    Task,
}

impl fmt::Display for MemoryScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryScope::Global => write!(f, "GLOBAL"),
            MemoryScope::Workspace => write!(f, "WORKSPACE"),
            MemoryScope::Project => write!(f, "PROJECT"),
            MemoryScope::Session => write!(f, "SESSION"),
            MemoryScope::Task => write!(f, "TASK"),
        }
    }
}

impl FromStr for MemoryScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "GLOBAL" => Ok(MemoryScope::Global),
            "WORKSPACE" => Ok(MemoryScope::Workspace),
            "PROJECT" => Ok(MemoryScope::Project),
            "SESSION" => Ok(MemoryScope::Session),
            "TASK" => Ok(MemoryScope::Task),
            other => Err(format!("Unknown memory scope: {}", other)),
        }
    }
}

/// Memory provenance sources. ModelDerived is untrusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemorySource {
    System,
    Company,
    UserExplicit,
    Observed,
    ToolDerived,
    ModelDerived,
    Imported,
    #[default]
    Unknown,
}

impl fmt::Display for MemorySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemorySource::System => write!(f, "SYSTEM"),
            MemorySource::Company => write!(f, "COMPANY"),
            MemorySource::UserExplicit => write!(f, "USER_EXPLICIT"),
            MemorySource::Observed => write!(f, "OBSERVED"),
            MemorySource::ToolDerived => write!(f, "TOOL_DERIVED"),
            MemorySource::ModelDerived => write!(f, "MODEL_DERIVED"),
            MemorySource::Imported => write!(f, "IMPORTED"),
            MemorySource::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl FromStr for MemorySource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "SYSTEM" => Ok(MemorySource::System),
            "COMPANY" => Ok(MemorySource::Company),
            "USER_EXPLICIT" | "USEREXPLICIT" | "USER" => Ok(MemorySource::UserExplicit),
            "OBSERVED" => Ok(MemorySource::Observed),
            "TOOL_DERIVED" | "TOOL" | "TOOLDERIVED" => Ok(MemorySource::ToolDerived),
            "MODEL_DERIVED" | "MODEL" | "MODELDERIVED" => Ok(MemorySource::ModelDerived),
            "IMPORTED" => Ok(MemorySource::Imported),
            "UNKNOWN" | "" => Ok(MemorySource::Unknown),
            other => Err(format!("Unknown memory source: {}", other)),
        }
    }
}

impl MemorySource {
    pub fn trust_rank(&self) -> u8 {
        match self {
            MemorySource::System => 100,
            MemorySource::Company => 80,
            MemorySource::UserExplicit => 70,
            MemorySource::Observed => 40,
            MemorySource::ToolDerived => 30,
            MemorySource::ModelDerived => 10,
            MemorySource::Imported => 10,
            MemorySource::Unknown => 0,
        }
    }

    pub fn is_high_trust(&self) -> bool {
        matches!(
            self,
            MemorySource::System | MemorySource::Company | MemorySource::UserExplicit
        )
    }
}

/// Stored memory item. Content is a structured fact/state summary — never a
/// hidden chain-of-thought transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub mem_type: MemoryType,
    pub scope: MemoryScope,
    pub workspace_id: Option<String>,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub content: String,
    pub classification: PrivacyClassification,
    pub source: MemorySource,
    pub confidence: f64,
    pub provenance: MemoryProvenance,
    pub created_at: u64,
    pub updated_at: u64,
    pub expires_at: Option<u64>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProvenance {
    pub source: MemorySource,
    pub actor: String,
    pub tool_id: Option<String>,
    pub model_id: Option<String>,
    pub imported_from: Option<String>,
    pub evidence: String,
}

impl Default for MemoryProvenance {
    fn default() -> Self {
        Self {
            source: MemorySource::Unknown,
            actor: "unknown".to_string(),
            tool_id: None,
            model_id: None,
            imported_from: None,
            evidence: String::new(),
        }
    }
}

/// Candidate memory awaiting approval. Model-generated facts stay candidates
/// until validated/approved; they never become policy automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCandidate {
    pub id: String,
    pub mem_type: MemoryType,
    pub scope: MemoryScope,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub content: String,
    pub classification: PrivacyClassification,
    pub source: MemorySource,
    pub confidence: f64,
    pub provenance: MemoryProvenance,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Scoped retrieval query. Classification ceiling + trust requirements enforced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub scope: Option<MemoryScope>,
    pub workspace_id: Option<String>,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub mem_type: Option<MemoryType>,
    pub classification_ceiling: PrivacyClassification,
    pub min_trust_rank: u8,
    pub allow_secret: bool,
    pub limit: usize,
    pub query_text: Option<String>,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            scope: None,
            workspace_id: None,
            project_id: None,
            session_id: None,
            task_id: None,
            mem_type: None,
            classification_ceiling: PrivacyClassification::Internal,
            min_trust_rank: 0,
            allow_secret: false,
            limit: 10,
            query_text: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    pub item: MemoryItem,
    pub relevance: f64,
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
