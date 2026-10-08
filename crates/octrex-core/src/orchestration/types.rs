use crate::ids::{SessionId, TaskId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported actions within a TaskStep
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepActionType {
    ThinkAnalyze,
    RetrieveContext,
    ReadFile,
    WriteFile,
    ExecuteTool,
    ModelCall,
    Verify,
    AskUser,
    Complete,
}

impl std::fmt::Display for StepActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepActionType::ThinkAnalyze => write!(f, "THINK_ANALYZE"),
            StepActionType::RetrieveContext => write!(f, "RETRIEVE_CONTEXT"),
            StepActionType::ReadFile => write!(f, "READ_FILE"),
            StepActionType::WriteFile => write!(f, "WRITE_FILE"),
            StepActionType::ExecuteTool => write!(f, "EXECUTE_TOOL"),
            StepActionType::ModelCall => write!(f, "MODEL_CALL"),
            StepActionType::Verify => write!(f, "VERIFY"),
            StepActionType::AskUser => write!(f, "ASK_USER"),
            StepActionType::Complete => write!(f, "COMPLETE"),
        }
    }
}

/// Status of an individual TaskStep
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepStatus {
    Pending,
    Executing,
    WaitingForUser,
    Completed,
    Failed,
    Skipped,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "PENDING"),
            StepStatus::Executing => write!(f, "EXECUTING"),
            StepStatus::WaitingForUser => write!(f, "WAITING_FOR_USER"),
            StepStatus::Completed => write!(f, "COMPLETED"),
            StepStatus::Failed => write!(f, "FAILED"),
            StepStatus::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// Individual step within a TaskPlan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub order: u32,
    pub objective: String,
    pub action_type: StepActionType,
    pub dependencies: Vec<String>,
    pub status: StepStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    pub requires_user_input: bool,
    pub verification_required: bool,
    pub inputs: Option<serde_json::Value>,
    pub outputs: Option<serde_json::Value>,
    pub failure_reason: Option<String>,
}

impl TaskStep {
    pub fn new(order: u32, objective: impl Into<String>, action_type: StepActionType) -> Self {
        Self {
            id: format!("step-{}", uuid::Uuid::new_v4().simple()),
            order,
            objective: objective.into(),
            action_type,
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            attempts: 0,
            max_attempts: 3,
            requires_user_input: false,
            verification_required: false,
            inputs: None,
            outputs: None,
            failure_reason: None,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_user_input_required(mut self, required: bool) -> Self {
        self.requires_user_input = required;
        self
    }

    pub fn with_verification(mut self, required: bool) -> Self {
        self.verification_required = required;
        self
    }

    pub fn with_inputs(mut self, inputs: serde_json::Value) -> Self {
        self.inputs = Some(inputs);
        self
    }
}

/// Status of a TaskPlan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlanStatus {
    Draft,
    Approved,
    Rejected,
    Executing,
    Completed,
    Failed,
    Replanned,
}

/// Execution plan for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub id: String,
    pub task_id: TaskId,
    pub objective: String,
    pub constraints: Vec<String>,
    pub steps: Vec<TaskStep>,
    pub current_step: usize,
    pub status: PlanStatus,
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl TaskPlan {
    pub fn new(task_id: TaskId, objective: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: format!("plan-{}", uuid::Uuid::new_v4().simple()),
            task_id,
            objective: objective.into(),
            constraints: Vec::new(),
            steps: Vec::new(),
            current_step: 0,
            status: PlanStatus::Draft,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Orchestrator state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrchestratorState {
    Created,
    Planning,
    PlanReady,
    Executing,
    WaitingForTool,
    WaitingForUser,
    Verifying,
    Retrying,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for OrchestratorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorState::Created => write!(f, "CREATED"),
            OrchestratorState::Planning => write!(f, "PLANNING"),
            OrchestratorState::PlanReady => write!(f, "PLAN_READY"),
            OrchestratorState::Executing => write!(f, "EXECUTING"),
            OrchestratorState::WaitingForTool => write!(f, "WAITING_FOR_TOOL"),
            OrchestratorState::WaitingForUser => write!(f, "WAITING_FOR_USER"),
            OrchestratorState::Verifying => write!(f, "VERIFYING"),
            OrchestratorState::Retrying => write!(f, "RETRYING"),
            OrchestratorState::Blocked => write!(f, "BLOCKED"),
            OrchestratorState::Completed => write!(f, "COMPLETED"),
            OrchestratorState::Failed => write!(f, "FAILED"),
            OrchestratorState::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

impl OrchestratorState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrchestratorState::Completed | OrchestratorState::Failed | OrchestratorState::Cancelled
        )
    }
}

/// Next decision from the execution loop
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionDecision {
    Continue,
    Retry { reason: String, attempt: u32 },
    AskUser { prompt: String },
    Block { reason: String },
    Complete { summary: String },
    Fail { error: String },
}

/// Request structure for Phase 12 Model Router integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub task_id: TaskId,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub purpose: String,
    pub context_requirements: Option<serde_json::Value>,
    pub capability_requirements: Vec<String>,
    pub latency_requirement: Option<String>,
    pub execution_constraints: HashMap<String, String>,
}

/// Model execution selection target returned by Phase 12 interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionTarget {
    pub provider_id: String,
    pub model_id: String,
    pub reasoning: String,
}
