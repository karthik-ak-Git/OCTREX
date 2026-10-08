use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::privacy::PrivacyClassification;
use crate::tools::ToolCapability;

/// Canonical source of a skill. Unknown fails closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkillSource {
    System,
    Company,
    BuiltIn,
    User,
    Imported,
    ModelGenerated,
    External,
    #[default]
    Unknown,
}

impl fmt::Display for SkillSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillSource::System => write!(f, "SYSTEM"),
            SkillSource::Company => write!(f, "COMPANY"),
            SkillSource::BuiltIn => write!(f, "BUILTIN"),
            SkillSource::User => write!(f, "USER"),
            SkillSource::Imported => write!(f, "IMPORTED"),
            SkillSource::ModelGenerated => write!(f, "MODEL_GENERATED"),
            SkillSource::External => write!(f, "EXTERNAL"),
            SkillSource::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl FromStr for SkillSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "SYSTEM" => Ok(SkillSource::System),
            "COMPANY" => Ok(SkillSource::Company),
            "BUILTIN" | "BUILT_IN" | "BUILT-IN" => Ok(SkillSource::BuiltIn),
            "USER" => Ok(SkillSource::User),
            "IMPORTED" => Ok(SkillSource::Imported),
            "MODEL_GENERATED" | "MODELGENERATED" | "MODEL" => Ok(SkillSource::ModelGenerated),
            "EXTERNAL" => Ok(SkillSource::External),
            "UNKNOWN" | "" => Ok(SkillSource::Unknown),
            other => Err(format!("Unknown skill source: {}", other)),
        }
    }
}

impl SkillSource {
    /// Explicit trust rank. Higher wins. Unknown = 0 (fail closed).
    pub fn trust_rank(&self) -> u8 {
        match self {
            SkillSource::System => 100,
            SkillSource::Company => 80,
            SkillSource::BuiltIn => 60,
            SkillSource::User => 50,
            SkillSource::Imported => 20,
            SkillSource::ModelGenerated => 10,
            SkillSource::External => 10,
            SkillSource::Unknown => 0,
        }
    }

    /// ModelGenerated / Imported / External are never automatically trusted.
    pub fn is_auto_trusted(&self) -> bool {
        matches!(
            self,
            SkillSource::System | SkillSource::Company | SkillSource::BuiltIn | SkillSource::User
        )
    }
}

/// Lifecycle status of a skill. Only Active skills are eligible for matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkillStatus {
    #[default]
    Draft,
    PendingValidation,
    Active,
    Disabled,
    Blocked,
}

impl fmt::Display for SkillStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillStatus::Draft => write!(f, "DRAFT"),
            SkillStatus::PendingValidation => write!(f, "PENDING_VALIDATION"),
            SkillStatus::Active => write!(f, "ACTIVE"),
            SkillStatus::Disabled => write!(f, "DISABLED"),
            SkillStatus::Blocked => write!(f, "BLOCKED"),
        }
    }
}

impl FromStr for SkillStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().trim() {
            "DRAFT" => Ok(SkillStatus::Draft),
            "PENDING_VALIDATION" => Ok(SkillStatus::PendingValidation),
            "ACTIVE" => Ok(SkillStatus::Active),
            "DISABLED" => Ok(SkillStatus::Disabled),
            "BLOCKED" => Ok(SkillStatus::Blocked),
            other => Err(format!("Unknown skill status: {}", other)),
        }
    }
}

/// Declarative step inside a skill's embedded workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkillStepKind {
    Skill,
    Tool,
    ModelOperation,
    FileOperation,
    Verification,
    UserInput,
}

impl fmt::Display for SkillStepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillStepKind::Skill => write!(f, "SKILL"),
            SkillStepKind::Tool => write!(f, "TOOL"),
            SkillStepKind::ModelOperation => write!(f, "MODEL_OPERATION"),
            SkillStepKind::FileOperation => write!(f, "FILE_OPERATION"),
            SkillStepKind::Verification => write!(f, "VERIFICATION"),
            SkillStepKind::UserInput => write!(f, "USER_INPUT"),
        }
    }
}

/// Declarative skill step. References are validated, never executed directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub id: String,
    pub kind: SkillStepKind,
    /// Tool id, file path, model op name, or nested skill id depending on kind.
    pub reference: String,
    pub description: String,
    pub required_capabilities: Vec<ToolCapability>,
    pub inputs: Option<serde_json::Value>,
    pub expected_output: Option<String>,
}

impl SkillStep {
    pub fn new(
        id: impl Into<String>,
        kind: SkillStepKind,
        reference: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            reference: reference.into(),
            description: description.into(),
            required_capabilities: Vec::new(),
            inputs: None,
            expected_output: None,
        }
    }
}

/// Full declarative skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub owner: String,
    pub source: SkillSource,
    pub status: SkillStatus,
    pub classification: PrivacyClassification,
    pub capabilities_required: Vec<ToolCapability>,
    pub allowed_tools: Vec<String>,
    pub workflow: Vec<SkillStep>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub verification_requirements: Vec<String>,
    pub provenance: SkillProvenance,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SkillDefinition {
    pub fn versioned_id(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
}

/// Provenance tracked for every skill version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProvenance {
    pub created_by: String,
    pub source: SkillSource,
    pub imported_from: Option<String>,
    pub approval_status: String,
    pub validation_status: String,
    pub tasks_used_in: Vec<String>,
    pub last_updated: u64,
}

impl Default for SkillProvenance {
    fn default() -> Self {
        Self {
            created_by: "unknown".to_string(),
            source: SkillSource::Unknown,
            imported_from: None,
            approval_status: "UNREVIEWED".to_string(),
            validation_status: "UNVALIDATED".to_string(),
            tasks_used_in: Vec::new(),
            last_updated: 0,
        }
    }
}

/// Runtime performance metrics. Metrics never grant capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMetrics {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub verification_passes: u64,
    pub verification_failures: u64,
    pub total_duration_ms: u64,
    pub common_failures: Vec<String>,
}

impl SkillMetrics {
    pub fn success_rate(&self) -> f64 {
        if self.total_runs == 0 {
            0.0
        } else {
            self.successful_runs as f64 / self.total_runs as f64
        }
    }

    pub fn average_duration_ms(&self) -> u64 {
        if self.total_runs == 0 {
            0
        } else {
            self.total_duration_ms / self.total_runs
        }
    }
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
