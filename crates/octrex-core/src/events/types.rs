use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    ApplicationReady,

    TaskCreated,
    TaskStarted,
    TaskPaused,
    TaskResumed,
    TaskCancelled,
    TaskCompleted,
    TaskFailed,

    StepStarted,
    StepCompleted,

    PrivacyCheckStarted,
    PrivacyDecision,

    ModelSelected,
    ModelStarted,
    ModelCompleted,
    ModelFailed,

    PermissionRequired,
    PermissionGranted,
    PermissionDenied,

    ToolStarted,
    ToolOutput,
    ToolCompleted,
    ToolFailed,

    FileRead,
    FileCreated,
    FileChanged,
    FileDeleted,

    TerminalStarted,
    TerminalOutput,
    TerminalExited,

    VerificationStarted,
    VerificationPassed,
    VerificationFailed,

    NetworkAllowed,
    NetworkBlocked,
    NetworkRequestStarted,
    NetworkPolicyEvaluationStarted,
    NetworkConsentRequired,
    NetworkRequestCompleted,
    NetworkRequestFailed,
    NetworkRedirectBlocked,
    NetworkSsrfBlocked,
    NetworkDnsBlocked,

    HardwareDetectionStarted,
    HardwareDetectionCompleted,
    HardwareDetectionFailed,
    HardwareProfileChanged,
    ModelCompatibilityChecked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreatedPayload {
    pub task_id: String,
    pub title: String,
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStartedPayload {
    pub task_id: String,
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedPayload {
    pub task_id: String,
    pub output_summary: String,
    pub execution_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailedPayload {
    pub task_id: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepPayload {
    pub step_index: usize,
    pub step_name: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyDecisionPayload {
    pub resource: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectedPayload {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequiredPayload {
    pub permission_type: String,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStartedPayload {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangedPayload {
    pub path: String,
    pub rel_path: String,
    pub action: String, // e.g. "created", "modified", "deleted"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputPayload {
    pub command: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResultPayload {
    pub check_name: String,
    pub passed: bool,
    pub output: String,
}
