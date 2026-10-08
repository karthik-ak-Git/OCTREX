use super::coordinator::OrchestratorCoordinator;
use super::errors::OrchestrationError;
use super::executor::ModelRouterInterface;
use super::limits::OrchestratorLimits;
use super::types::{ExecutionDecision, TaskPlan, TaskStep};
use crate::context::ContextService;
use crate::db::manager::DatabaseManager;
use crate::events::bus::EventBus;
use crate::filesystem::FilesystemSecurityService;
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::models::{ModelRegistry, ModelRuntime};
use crate::privacy::PrivacyGate;
use crate::task::{Task, TaskRegistry};
use crate::tools::ToolRuntime;
use std::sync::Arc;

#[derive(Clone)]
pub struct OrchestrationService {
    coordinator: Arc<OrchestratorCoordinator>,
}

impl OrchestrationService {
    pub fn new(
        limits: OrchestratorLimits,
        task_registry: Arc<TaskRegistry>,
        db: Arc<DatabaseManager>,
        event_bus: Arc<EventBus>,
        model_runtime: Arc<ModelRuntime>,
        model_registry: Arc<ModelRegistry>,
        tool_runtime: Arc<ToolRuntime>,
        filesystem_security: Arc<FilesystemSecurityService>,
        context_engine: Arc<ContextService>,
        privacy_gate: Arc<PrivacyGate>,
    ) -> Self {
        let coordinator = Arc::new(OrchestratorCoordinator::new(
            limits,
            task_registry,
            db,
            event_bus,
            model_runtime,
            model_registry,
            tool_runtime,
            filesystem_security,
            context_engine,
            privacy_gate,
        ));

        Self { coordinator }
    }

    pub fn with_router_interface(self, router: Arc<dyn ModelRouterInterface>) -> Self {
        let coordinator = Arc::new((*self.coordinator).clone().with_router_interface(router));
        Self { coordinator }
    }

    pub fn with_verification_engine(
        self,
        engine: Arc<crate::verification::VerificationEngine>,
    ) -> Self {
        let coordinator = Arc::new((*self.coordinator).clone().with_verification_engine(engine));
        Self { coordinator }
    }

    pub fn coordinator(&self) -> &OrchestratorCoordinator {
        &self.coordinator
    }

    pub fn create_task_and_plan(
        &self,
        objective: impl Into<String>,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
    ) -> Result<(Task, TaskPlan), OrchestrationError> {
        self.coordinator
            .create_task_and_plan(objective, session_id, workspace_id)
    }

    pub async fn start_task(
        &self,
        task_id: &TaskId,
        workspace_path: Option<String>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        self.coordinator.start_task(task_id, workspace_path).await
    }

    pub fn pause_task(&self, task_id: &TaskId) -> Result<bool, OrchestrationError> {
        self.coordinator.pause_task(task_id)
    }

    pub async fn resume_task(
        &self,
        task_id: &TaskId,
        workspace_path: Option<String>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        self.coordinator.resume_task(task_id, workspace_path).await
    }

    pub fn cancel_task(&self, task_id: &TaskId) -> Result<bool, OrchestrationError> {
        self.coordinator.cancel_task(task_id)
    }

    pub async fn submit_user_input(
        &self,
        task_id: &TaskId,
        user_input: &str,
        workspace_path: Option<String>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        self.coordinator
            .submit_user_input(task_id, user_input, workspace_path)
            .await
    }

    pub fn replan_task(
        &self,
        task_id: &TaskId,
        revised_objective: &str,
    ) -> Result<TaskPlan, OrchestrationError> {
        self.coordinator.replan_task(task_id, revised_objective)
    }

    pub fn get_plan(&self, task_id: &TaskId) -> Option<TaskPlan> {
        self.coordinator.get_plan(task_id)
    }

    pub fn get_steps(&self, task_id: &TaskId) -> Vec<TaskStep> {
        self.coordinator
            .get_plan(task_id)
            .map(|p| p.steps)
            .unwrap_or_default()
    }
}
