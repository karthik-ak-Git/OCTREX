use serde::{Deserialize, Serialize};

/// Maximum bounded stdout/stderr characters retained per command evidence.
pub const MAX_COMMAND_OUTPUT_CHARS: usize = 2000;
/// Maximum bounded file preview characters retained for content checks.
pub const MAX_FILE_PREVIEW_CHARS: usize = 2000;

pub fn bound_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}…[truncated]", truncated)
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Provenance-carrying evidence. Raw secrets and full file contents are never stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationEvidence {
    File {
        evidence_id: String,
        path_ref: String,
        operation: String,
        exists: bool,
        size: Option<u64>,
        classification: String,
        timestamp: u64,
    },
    Tool {
        evidence_id: String,
        tool_id: String,
        status: String,
        duration_ms: u128,
        timestamp: u64,
    },
    Command {
        evidence_id: String,
        command_ref: String,
        exit_code: Option<i32>,
        duration_ms: u128,
        stdout_summary: String,
        stderr_summary: String,
        status: String,
        timestamp: u64,
    },
    Artifact {
        evidence_id: String,
        artifact_id: Option<String>,
        path_ref: String,
        valid: bool,
        size: Option<u64>,
        timestamp: u64,
    },
    Test {
        evidence_id: String,
        suite: String,
        passed: u32,
        failed: u32,
        total: u32,
        timestamp: u64,
    },
    TaskState {
        evidence_id: String,
        task_id: String,
        steps_total: usize,
        steps_completed: usize,
        timestamp: u64,
    },
    UserRequirement {
        evidence_id: String,
        requirement: String,
        satisfied: bool,
        timestamp: u64,
    },
}

impl VerificationEvidence {
    pub fn id(&self) -> &str {
        match self {
            VerificationEvidence::File { evidence_id, .. } => evidence_id,
            VerificationEvidence::Tool { evidence_id, .. } => evidence_id,
            VerificationEvidence::Command { evidence_id, .. } => evidence_id,
            VerificationEvidence::Artifact { evidence_id, .. } => evidence_id,
            VerificationEvidence::Test { evidence_id, .. } => evidence_id,
            VerificationEvidence::TaskState { evidence_id, .. } => evidence_id,
            VerificationEvidence::UserRequirement { evidence_id, .. } => evidence_id,
        }
    }

    pub fn file(
        path_ref: impl Into<String>,
        operation: impl Into<String>,
        exists: bool,
        size: Option<u64>,
        classification: impl Into<String>,
    ) -> Self {
        VerificationEvidence::File {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            path_ref: path_ref.into(),
            operation: operation.into(),
            exists,
            size,
            classification: classification.into(),
            timestamp: now_millis(),
        }
    }

    pub fn tool(tool_id: impl Into<String>, status: impl Into<String>, duration_ms: u128) -> Self {
        VerificationEvidence::Tool {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            tool_id: tool_id.into(),
            status: status.into(),
            duration_ms,
            timestamp: now_millis(),
        }
    }

    pub fn command(
        command_ref: impl Into<String>,
        exit_code: Option<i32>,
        duration_ms: u128,
        stdout: &str,
        stderr: &str,
    ) -> Self {
        let status = match exit_code {
            Some(0) => "SUCCEEDED",
            Some(_) => "FAILED",
            None => "UNKNOWN",
        };
        VerificationEvidence::Command {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            command_ref: command_ref.into(),
            exit_code,
            duration_ms,
            stdout_summary: bound_text(stdout, MAX_COMMAND_OUTPUT_CHARS),
            stderr_summary: bound_text(stderr, MAX_COMMAND_OUTPUT_CHARS),
            status: status.to_string(),
            timestamp: now_millis(),
        }
    }

    pub fn artifact(
        artifact_id: Option<String>,
        path_ref: impl Into<String>,
        valid: bool,
        size: Option<u64>,
    ) -> Self {
        VerificationEvidence::Artifact {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            artifact_id,
            path_ref: path_ref.into(),
            valid,
            size,
            timestamp: now_millis(),
        }
    }

    pub fn test(suite: impl Into<String>, passed: u32, failed: u32, total: u32) -> Self {
        VerificationEvidence::Test {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            suite: suite.into(),
            passed,
            failed,
            total,
            timestamp: now_millis(),
        }
    }

    pub fn task_state(
        task_id: impl Into<String>,
        steps_total: usize,
        steps_completed: usize,
    ) -> Self {
        VerificationEvidence::TaskState {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            task_id: task_id.into(),
            steps_total,
            steps_completed,
            timestamp: now_millis(),
        }
    }

    pub fn user_requirement(requirement: impl Into<String>, satisfied: bool) -> Self {
        VerificationEvidence::UserRequirement {
            evidence_id: format!("ev-{}", uuid::Uuid::new_v4().simple()),
            requirement: requirement.into(),
            satisfied,
            timestamp: now_millis(),
        }
    }
}
