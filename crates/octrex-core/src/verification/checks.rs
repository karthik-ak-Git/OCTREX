use serde::{Deserialize, Serialize};

/// Declarative expectation for a file inside the authorized workspace.
/// Paths are always workspace-relative and resolved through
/// FilesystemSecurityService. Absolute paths are rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedFile {
    pub rel_path: String,
    pub must_exist: bool,
    pub must_not_be_empty: Option<bool>,
    pub expected_extension: Option<String>,
    pub expected_contains: Option<String>,
    pub must_be_modified_after_ms: Option<u64>,
}

impl ExpectedFile {
    pub fn exists(rel_path: impl Into<String>) -> Self {
        Self {
            rel_path: rel_path.into(),
            must_exist: true,
            must_not_be_empty: None,
            expected_extension: None,
            expected_contains: None,
            must_be_modified_after_ms: None,
        }
    }
}

/// Declarative expectation for a registered artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedArtifact {
    pub name: String,
    pub artifact_type: Option<String>,
    pub max_size_bytes: Option<u64>,
}

/// Structured command/build/test execution evidence supplied by the
/// orchestrator from an authorized execution mechanism.
///
/// The verification engine never executes commands itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvidenceInput {
    pub command_ref: String,
    pub kind: CommandKind,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandKind {
    Build,
    Test,
    Command,
}

impl Default for CommandKind {
    fn default() -> Self {
        CommandKind::Command
    }
}

/// Tool execution evidence. The tool must already have been executed
/// through ToolRuntime; verification only inspects the recorded outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEvidenceInput {
    pub tool_id: String,
    pub status: String,
    pub duration_ms: u128,
    pub result: serde_json::Value,
    pub expected_result_field: Option<String>,
}

/// Step completion evidence from the orchestrator's authoritative step store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvidenceInput {
    pub step_id: String,
    pub status: String,
    pub has_error: bool,
}

/// Task-level constraint to verify independently of model output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraint {
    pub constraint_type: String,
    pub description: String,
    /// Observed authoritative fact, e.g. "local_only", "no_cloud", "workspace_only".
    pub observed: Option<String>,
}

/// Explicit user requirement preserved structurally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequirement {
    pub requirement: String,
    pub requirement_type: String,
    /// Authoritative observed satisfaction signal (not a model claim).
    pub observed_satisfied: Option<bool>,
}

/// Full verification request for a task (optionally scoped to one step).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskVerificationRequest {
    pub task_id: String,
    pub step_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    #[serde(default)]
    pub expected_files: Vec<ExpectedFile>,
    #[serde(default)]
    pub expected_artifacts: Vec<ExpectedArtifact>,
    #[serde(default)]
    pub commands: Vec<CommandEvidenceInput>,
    #[serde(default)]
    pub tools: Vec<ToolEvidenceInput>,
    #[serde(default)]
    pub steps: Vec<StepEvidenceInput>,
    #[serde(default)]
    pub constraints: Vec<TaskConstraint>,
    #[serde(default)]
    pub requirements: Vec<UserRequirement>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_value: Option<serde_json::Value>,
    /// A model claim such as "done". Recorded for audit, never trusted.
    #[serde(default)]
    pub model_claim: Option<String>,
    /// Whether a supplemental local/free AI advisory check may run.
    #[serde(default = "default_true")]
    pub allow_ai_advisory: bool,
    #[serde(default)]
    pub repair_attempt: u32,
}

fn default_true() -> bool {
    true
}

impl TaskVerificationRequest {
    pub fn for_task(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            step_id: None,
            workspace_id: None,
            session_id: None,
            expected_files: vec![],
            expected_artifacts: vec![],
            commands: vec![],
            tools: vec![],
            steps: vec![],
            constraints: vec![],
            requirements: vec![],
            output_schema: None,
            output_value: None,
            model_claim: None,
            allow_ai_advisory: true,
            repair_attempt: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.task_id.trim().is_empty() {
            return Err("task_id must not be empty".to_string());
        }
        for f in &self.expected_files {
            if f.rel_path.trim().is_empty() {
                return Err("expected file rel_path must not be empty".to_string());
            }
            if f.rel_path.starts_with('/') || f.rel_path.starts_with('\\') {
                return Err(format!(
                    "expected file '{}' must be workspace-relative, not absolute",
                    f.rel_path
                ));
            }
            if f.rel_path.contains("..") {
                return Err(format!(
                    "expected file '{}' must not contain '..'",
                    f.rel_path
                ));
            }
        }
        for c in &self.commands {
            if c.command_ref.trim().is_empty() {
                return Err("command_ref must not be empty".to_string());
            }
        }
        for t in &self.tools {
            if t.tool_id.trim().is_empty() {
                return Err("tool_id must not be empty".to_string());
            }
        }
        Ok(())
    }
}
