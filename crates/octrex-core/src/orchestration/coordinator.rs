use super::cancellation::CancellationManager;
use super::errors::OrchestrationError;
use super::executor::{DefaultModelRouterInterface, ModelRouterInterface, TaskExecutor};
use super::limits::OrchestratorLimits;
use super::planner::TaskPlanner;
use super::recovery::TaskRecoveryManager;
use super::types::{ExecutionDecision, OrchestratorState, StepStatus, TaskPlan};
use crate::context::ContextService;
use crate::db::manager::DatabaseManager;
use crate::events::bus::EventBus;
use crate::filesystem::FilesystemSecurityService;
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::models::{ModelRegistry, ModelRuntime};
use crate::privacy::PrivacyGate;
use crate::task::{Task, TaskRegistry, TaskStatus};
use crate::tools::ToolRuntime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct OrchestratorCoordinator {
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
    router_interface: Arc<dyn ModelRouterInterface>,
    verification_engine: Option<Arc<crate::verification::VerificationEngine>>,
    cancellation_mgr: CancellationManager,
    active_plans: Arc<RwLock<HashMap<TaskId, TaskPlan>>>,
}

impl OrchestratorCoordinator {
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
        Self {
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
            router_interface: Arc::new(DefaultModelRouterInterface),
            verification_engine: None,
            cancellation_mgr: CancellationManager::new(),
            active_plans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_verification_engine(
        mut self,
        engine: Arc<crate::verification::VerificationEngine>,
    ) -> Self {
        self.verification_engine = Some(engine);
        self
    }

    pub fn with_router_interface(mut self, router: Arc<dyn ModelRouterInterface>) -> Self {
        self.router_interface = router;
        self
    }

    pub fn cancellation_manager(&self) -> &CancellationManager {
        &self.cancellation_mgr
    }

    /// Create task and initial execution plan
    pub fn create_task_and_plan(
        &self,
        objective: impl Into<String>,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
    ) -> Result<(Task, TaskPlan), OrchestrationError> {
        let obj_str = objective.into();
        let task = self
            .task_registry
            .create_task(&obj_str, None, session_id, workspace_id);

        let plan = TaskPlanner::create_plan(
            task.id.clone(),
            &obj_str,
            &self.limits,
            Some(self.tool_runtime.registry()),
        )?;

        {
            let mut guard = self.active_plans.write().unwrap();
            guard.insert(task.id.clone(), plan.clone());
        }

        let _ = TaskRecoveryManager::save_checkpoint(&self.db, &plan, OrchestratorState::Created);

        Ok((task, plan))
    }

    /// Start execution of a task in the background loop
    pub async fn start_task(
        &self,
        task_id: &TaskId,
        workspace_path: Option<String>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        let task =
            self.task_registry
                .get_task(task_id)
                .ok_or_else(|| OrchestrationError::Internal {
                    message: format!("Task '{}' not found", task_id),
                })?;

        let plan = {
            let guard = self.active_plans.read().unwrap();
            if let Some(p) = guard.get(task_id) {
                p.clone()
            } else {
                TaskRecoveryManager::load_plan(&self.db, task_id)?.ok_or_else(|| {
                    OrchestrationError::InvalidPlan {
                        reason: format!("No plan found for task '{}'", task_id),
                    }
                })?
            }
        };

        let mut executor = TaskExecutor::new(
            self.limits.clone(),
            self.task_registry.clone(),
            self.db.clone(),
            self.event_bus.clone(),
            self.model_runtime.clone(),
            self.model_registry.clone(),
            self.tool_runtime.clone(),
            self.filesystem_security.clone(),
            self.context_engine.clone(),
            self.privacy_gate.clone(),
            self.cancellation_mgr.clone(),
        )
        .with_router_interface(self.router_interface.clone());
        if let Some(engine) = self.verification_engine.clone() {
            executor = executor.with_verification_engine(engine);
        }

        executor
            .execute_loop(
                task_id.clone(),
                task.session_id,
                task.workspace_id,
                workspace_path,
                plan,
            )
            .await
    }

    /// Pause task execution
    pub fn pause_task(&self, task_id: &TaskId) -> Result<bool, OrchestrationError> {
        let task = self
            .task_registry
            .update_status(task_id, TaskStatus::Paused, None);
        Ok(task.is_some())
    }

    /// Resume task execution from checkpoint
    pub async fn resume_task(
        &self,
        task_id: &TaskId,
        workspace_path: Option<String>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        self.start_task(task_id, workspace_path).await
    }

    /// Cancel active task execution
    pub fn cancel_task(&self, task_id: &TaskId) -> Result<bool, OrchestrationError> {
        let cancelled = self.cancellation_mgr.cancel_task(task_id);
        self.task_registry
            .update_status(task_id, TaskStatus::Cancelled, None);
        Ok(cancelled)
    }

    /// Submit user input for a task paused in `WaitingForUser`
    pub async fn submit_user_input(
        &self,
        task_id: &TaskId,
        user_input: &str,
        workspace_path: Option<String>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        let mut plan = {
            let guard = self.active_plans.read().unwrap();
            guard
                .get(task_id)
                .cloned()
                .ok_or_else(|| OrchestrationError::InvalidPlan {
                    reason: format!("Plan not found for task '{}'", task_id),
                })?
        };

        if let Some(step) = plan.steps.get_mut(plan.current_step) {
            if step.status == StepStatus::WaitingForUser {
                step.inputs = Some(serde_json::json!({ "user_response": user_input }));
                step.status = StepStatus::Completed;
                plan.current_step += 1;
            }
        }

        {
            let mut guard = self.active_plans.write().unwrap();
            guard.insert(task_id.clone(), plan.clone());
        }

        self.start_task(task_id, workspace_path).await
    }

    /// Replan a task with a revised objective or constraints while preserving finished steps
    pub fn replan_task(
        &self,
        task_id: &TaskId,
        revised_objective: &str,
    ) -> Result<TaskPlan, OrchestrationError> {
        let existing_plan = {
            let guard = self.active_plans.read().unwrap();
            guard
                .get(task_id)
                .cloned()
                .ok_or_else(|| OrchestrationError::InvalidPlan {
                    reason: format!("Plan not found for task '{}'", task_id),
                })?
        };

        let mut new_plan = TaskPlanner::create_plan(
            task_id.clone(),
            revised_objective,
            &self.limits,
            Some(self.tool_runtime.registry()),
        )?;

        new_plan.version = existing_plan.version + 1;

        {
            let mut guard = self.active_plans.write().unwrap();
            guard.insert(task_id.clone(), new_plan.clone());
        }

        let _ =
            TaskRecoveryManager::save_checkpoint(&self.db, &new_plan, OrchestratorState::PlanReady);

        Ok(new_plan)
    }

    /// Get current plan for a task
    pub fn get_plan(&self, task_id: &TaskId) -> Option<TaskPlan> {
        let guard = self.active_plans.read().unwrap();
        if let Some(p) = guard.get(task_id) {
            Some(p.clone())
        } else {
            TaskRecoveryManager::load_plan(&self.db, task_id)
                .ok()
                .flatten()
        }
    }
}
