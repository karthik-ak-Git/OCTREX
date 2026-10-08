use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    ApplicationReady,

    TaskCreated,
    TaskPlanningStarted,
    TaskPlanReady,
    TaskStarted,
    TaskPaused,
    TaskResumed,
    TaskRetryStarted,
    TaskReplanned,
    TaskWaitingForUser,
    TaskBlocked,
    TaskCancelled,
    TaskCompleted,
    TaskFailed,

    StepStarted,
    StepCompleted,
    StepFailed,

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
    ToolBlocked,

    FileRead,
    FileCreated,
    FileChanged,
    FileDeleted,

    TerminalStarted,
    TerminalOutput,
    TerminalExited,

    VerificationStarted,
    VerificationCheckCompleted,
    VerificationPassed,
    VerificationFailed,
    VerificationBlocked,
    RepairStarted,
    CompletionGateEvaluated,
    TaskVerificationAdvisory,

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

    ContextBuildStarted,
    ContextBuildCompleted,
    ContextItemAdded,
    ContextItemExcluded,
    ContextCompactionStarted,
    ContextCompactionCompleted,
    ContextBudgetWarning,
    ContextBudgetExceeded,
    ContextCheckpointCreated,
    ContextRestored,

    RoutingStarted,
    RoutingCandidatesEvaluated,
    RoutingModelSelected,
    RoutingBlocked,
    RoutingNoCompatibleModel,
    RoutingRequireUserSelection,

    SkillCreated,
    SkillValidated,
    SkillEnabled,
    SkillDisabled,
    WorkflowCreated,
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    MemoryCandidateCreated,
    MemoryApproved,
    MemoryRejected,
    MemoryUpdated,
    MemoryDeleted,
    MemoryRetrieved,

    DocumentImported,
    DocumentParsed,
    DocumentClassified,
    DocumentChunked,
    DocumentRetrievalPerformed,
    ArtifactCreated,
    ArtifactVerified,
    ArtifactVerificationFailed,

    LocalRuntimeDiscovered,
    LocalRuntimeRegistered,
    LocalRuntimeHealthChanged,
    LocalModelDiscovered,
    LocalModelRegistered,
    LocalModelLoaded,
    LocalModelUnloaded,
    LocalInferenceStarted,
    LocalInferenceCompleted,
    LocalInferenceFailed,
    LocalInferenceCancelled,
    LocalModelDownloadStarted,
    LocalModelDownloadCompleted,
    LocalModelDownloadFailed,
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
