use serde::{Deserialize, Serialize};

/// Terminal verification status for a run, step, or task.
///
/// SECURITY INVARIANT: `Unknown` MUST NOT be coerced to `Pass`.
/// For security-sensitive checks, `Unknown` is evaluated as Block/Fail
/// by `VerificationPolicy` and `CompletionGate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    Pass,
    PassWithWarnings,
    Fail,
    Blocked,
    Unknown,
}

impl VerificationStatus {
    pub fn is_pass(&self) -> bool {
        matches!(
            self,
            VerificationStatus::Pass | VerificationStatus::PassWithWarnings
        )
    }

    pub fn is_terminal_failure(&self) -> bool {
        matches!(self, VerificationStatus::Fail | VerificationStatus::Blocked)
    }

    /// Fail-closed sanitizer: Unknown never becomes Pass here.
    /// Callers must resolve Unknown explicitly via policy.
    pub fn sanitize_unknown(&self) -> Self {
        match self {
            VerificationStatus::Unknown => VerificationStatus::Blocked,
            other => *other,
        }
    }
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationStatus::Pass => write!(f, "PASS"),
            VerificationStatus::PassWithWarnings => write!(f, "PASS_WITH_WARNINGS"),
            VerificationStatus::Fail => write!(f, "FAIL"),
            VerificationStatus::Blocked => write!(f, "BLOCKED"),
            VerificationStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl std::str::FromStr for VerificationStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PASS" => Ok(VerificationStatus::Pass),
            "PASS_WITH_WARNINGS" => Ok(VerificationStatus::PassWithWarnings),
            "FAIL" => Ok(VerificationStatus::Fail),
            "BLOCKED" => Ok(VerificationStatus::Blocked),
            "UNKNOWN" => Ok(VerificationStatus::Unknown),
            other => Err(format!("Unknown verification status: {}", other)),
        }
    }
}

/// Individual check outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Passed,
    Warning,
    Failed,
    Blocked,
    Unknown,
    Skipped,
}

impl CheckStatus {
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckStatus::Passed | CheckStatus::Warning)
    }
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Passed => write!(f, "PASSED"),
            CheckStatus::Warning => write!(f, "WARNING"),
            CheckStatus::Failed => write!(f, "FAILED"),
            CheckStatus::Blocked => write!(f, "BLOCKED"),
            CheckStatus::Unknown => write!(f, "UNKNOWN"),
            CheckStatus::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// Severity of a check. Blocking/Critical failures always fail the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckSeverity {
    Info,
    Warning,
    Critical,
    Blocking,
}

impl std::fmt::Display for CheckSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckSeverity::Info => write!(f, "INFO"),
            CheckSeverity::Warning => write!(f, "WARNING"),
            CheckSeverity::Critical => write!(f, "CRITICAL"),
            CheckSeverity::Blocking => write!(f, "BLOCKING"),
        }
    }
}

/// Closed set of verification check types.
///
/// No arbitrary check type may bypass security: unknown types are rejected
/// at request validation time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationCheckType {
    FileExists,
    FileContent,
    FileModified,
    FileCreated,
    ArtifactExists,
    ArtifactValid,
    CommandSucceeded,
    TestPassed,
    BuildPassed,
    ToolSucceeded,
    OutputSchema,
    RequiredField,
    ConstraintSatisfied,
    StepCompleted,
    DependencySatisfied,
    UserRequirement,
    NoBlockingError,
}

impl std::fmt::Display for VerificationCheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VerificationCheckType::FileExists => "FILE_EXISTS",
            VerificationCheckType::FileContent => "FILE_CONTENT",
            VerificationCheckType::FileModified => "FILE_MODIFIED",
            VerificationCheckType::FileCreated => "FILE_CREATED",
            VerificationCheckType::ArtifactExists => "ARTIFACT_EXISTS",
            VerificationCheckType::ArtifactValid => "ARTIFACT_VALID",
            VerificationCheckType::CommandSucceeded => "COMMAND_SUCCEEDED",
            VerificationCheckType::TestPassed => "TEST_PASSED",
            VerificationCheckType::BuildPassed => "BUILD_PASSED",
            VerificationCheckType::ToolSucceeded => "TOOL_SUCCEEDED",
            VerificationCheckType::OutputSchema => "OUTPUT_SCHEMA",
            VerificationCheckType::RequiredField => "REQUIRED_FIELD",
            VerificationCheckType::ConstraintSatisfied => "CONSTRAINT_SATISFIED",
            VerificationCheckType::StepCompleted => "STEP_COMPLETED",
            VerificationCheckType::DependencySatisfied => "DEPENDENCY_SATISFIED",
            VerificationCheckType::UserRequirement => "USER_REQUIREMENT",
            VerificationCheckType::NoBlockingError => "NO_BLOCKING_ERROR",
        };
        write!(f, "{}", s)
    }
}

/// A single executed verification check with redacted evidence reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub id: String,
    pub name: String,
    pub check_type: VerificationCheckType,
    pub status: CheckStatus,
    pub severity: CheckSeverity,
    pub expected: String,
    pub actual: String,
    pub evidence_ref: Option<String>,
    pub message: String,
}

impl VerificationCheck {
    pub fn passed(
        name: impl Into<String>,
        check_type: VerificationCheckType,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("chk-{}", uuid::Uuid::new_v4().simple()),
            name: name.into(),
            check_type,
            status: CheckStatus::Passed,
            severity: CheckSeverity::Info,
            expected: expected.into(),
            actual: actual.into(),
            evidence_ref: None,
            message: "Check passed".to_string(),
        }
    }

    pub fn with_severity(mut self, severity: CheckSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_evidence(mut self, evidence_ref: impl Into<String>) -> Self {
        self.evidence_ref = Some(evidence_ref.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn failed(
        name: impl Into<String>,
        check_type: VerificationCheckType,
        severity: CheckSeverity,
        expected: impl Into<String>,
        actual: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("chk-{}", uuid::Uuid::new_v4().simple()),
            name: name.into(),
            check_type,
            status: if severity == CheckSeverity::Blocking {
                CheckStatus::Blocked
            } else {
                CheckStatus::Failed
            },
            severity,
            expected: expected.into(),
            actual: actual.into(),
            evidence_ref: None,
            message: message.into(),
        }
    }

    pub fn unknown(
        name: impl Into<String>,
        check_type: VerificationCheckType,
        severity: CheckSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("chk-{}", uuid::Uuid::new_v4().simple()),
            name: name.into(),
            check_type,
            status: CheckStatus::Unknown,
            severity,
            expected: "determinate evidence".to_string(),
            actual: "unknown".to_string(),
            evidence_ref: None,
            message: message.into(),
        }
    }

    pub fn warning(
        name: impl Into<String>,
        check_type: VerificationCheckType,
        expected: impl Into<String>,
        actual: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("chk-{}", uuid::Uuid::new_v4().simple()),
            name: name.into(),
            check_type,
            status: CheckStatus::Warning,
            severity: CheckSeverity::Warning,
            expected: expected.into(),
            actual: actual.into(),
            evidence_ref: None,
            message: message.into(),
        }
    }
}

/// Structured verification result returned to the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verification_id: String,
    pub task_id: String,
    pub step_id: Option<String>,
    pub status: VerificationStatus,
    pub checks: Vec<VerificationCheck>,
    pub evidence_refs: Vec<String>,
    pub warnings: Vec<String>,
    pub failures: Vec<String>,
    /// 0.0..=1.0 authoritative-evidence confidence. Model claims never raise this.
    pub confidence: f64,
    pub repair_attempt: u32,
    pub created_at: u64,
}

impl VerificationResult {
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|c| c.status.is_pass()).count()
    }

    pub fn failed_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    CheckStatus::Failed | CheckStatus::Blocked | CheckStatus::Unknown
                )
            })
            .count()
    }
}

/// Completion gate decision consumed by the Phase 11 orchestrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompletionDecision {
    AllowCompletion,
    RequireRepair,
    RequireUser,
    Blocked,
}

impl std::fmt::Display for CompletionDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionDecision::AllowCompletion => write!(f, "ALLOW_COMPLETION"),
            CompletionDecision::RequireRepair => write!(f, "REQUIRE_REPAIR"),
            CompletionDecision::RequireUser => write!(f, "REQUIRE_USER"),
            CompletionDecision::Blocked => write!(f, "BLOCKED"),
        }
    }
}

/// Orchestrator action derived from a verification result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationAction {
    Complete,
    Retry,
    Repair,
    Replan,
    AskUser,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIssue {
    pub path: String,
    pub expected: String,
    pub actual: String,
}
