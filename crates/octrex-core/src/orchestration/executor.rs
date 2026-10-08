use super::cancellation::CancellationManager;
use super::completion::{StandardVerificationService, VerificationService};
use super::errors::OrchestrationError;
use super::events::OrchestrationEventPublisher;
use super::limits::OrchestratorLimits;
use super::plan::PlanValidator;
use super::recovery::TaskRecoveryManager;
use super::retry::RetryManager;
use super::state::OrchestratorStateMachine;
use super::step::StepExecutor;
use super::types::{
    ExecutionDecision, ExecutionRequest, ModelSelectionTarget, OrchestratorState, PlanStatus,
    StepActionType, StepStatus, TaskPlan,
};
use crate::context::ContextService;
use crate::db::manager::DatabaseManager;
use crate::events::{bus::EventBus, types::EventType};
use crate::filesystem::FilesystemSecurityService;
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::models::{ModelRegistry, ModelRuntime};
use crate::privacy::PrivacyGate;
use crate::task::{TaskRegistry, TaskStatus};
use crate::tools::ToolRuntime;
use crate::verification::{
    ExpectedFile, StepEvidenceInput, TaskConstraint, TaskVerificationRequest, VerificationEngine,
};
use std::sync::Arc;

/// Phase 12 Model Router Interface trait definition
pub trait ModelRouterInterface: Send + Sync {
    fn select_target(
        &self,
        req: &ExecutionRequest,
        model_registry: &ModelRegistry,
    ) -> Result<ModelSelectionTarget, OrchestrationError>;
}

/// Default implementation routing through ModelRegistry
pub struct DefaultModelRouterInterface;

impl ModelRouterInterface for DefaultModelRouterInterface {
    fn select_target(
        &self,
        _req: &ExecutionRequest,
        model_registry: &ModelRegistry,
    ) -> Result<ModelSelectionTarget, OrchestrationError> {
        let models = model_registry.list_models();
        if let Some(m) = models.first() {
            Ok(ModelSelectionTarget {
                provider_id: m.provider_id.clone(),
                model_id: m.id.clone(),
                reasoning: "Selected registered model target".to_string(),
            })
        } else {
            Ok(ModelSelectionTarget {
                provider_id: "opencode".to_string(),
                model_id: "opencode-free-router".to_string(),
                reasoning: "Fallback default provider target".to_string(),
            })
        }
    }
}

pub struct TaskExecutor {
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
    verifier: Arc<dyn VerificationService>,
    verification_engine: Option<Arc<VerificationEngine>>,
    cancellation_mgr: CancellationManager,
}

impl TaskExecutor {
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
        cancellation_mgr: CancellationManager,
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
            verifier: Arc::new(StandardVerificationService::new()),
            verification_engine: None,
            cancellation_mgr,
        }
    }

    pub fn with_verification_engine(mut self, engine: Arc<VerificationEngine>) -> Self {
        self.verification_engine = Some(engine);
        self
    }

    pub fn with_router_interface(mut self, router: Arc<dyn ModelRouterInterface>) -> Self {
        self.router_interface = router;
        self
    }

    pub fn with_verifier(mut self, verifier: Arc<dyn VerificationService>) -> Self {
        self.verifier = verifier;
        self
    }

    /// Execute the autonomous orchestration loop until task reaches a terminal state or requires user input
    pub async fn execute_loop(
        &self,
        task_id: TaskId,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
        workspace_path: Option<String>,
        mut plan: TaskPlan,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        let mut state_machine = OrchestratorStateMachine::new();
        state_machine.transition_to(OrchestratorState::Planning)?;

        OrchestrationEventPublisher::publish_task_event(
            &self.event_bus,
            EventType::TaskPlanningStarted,
            &task_id,
            serde_json::json!({ "objective": plan.objective }),
        );

        // Validate plan
        PlanValidator::validate(&plan, &self.limits, Some(self.tool_runtime.registry()))?;

        state_machine.transition_to(OrchestratorState::PlanReady)?;
        plan.status = PlanStatus::Approved;

        OrchestrationEventPublisher::publish_task_event(
            &self.event_bus,
            EventType::TaskPlanReady,
            &task_id,
            serde_json::json!({ "total_steps": plan.steps.len(), "plan_id": plan.id }),
        );

        // Save initial plan checkpoint
        let _ = TaskRecoveryManager::save_checkpoint(&self.db, &plan, OrchestratorState::PlanReady);

        state_machine.transition_to(OrchestratorState::Executing)?;
        self.task_registry
            .update_status(&task_id, TaskStatus::Executing, None);

        let mut model_call_count = 0u32;

        while plan.current_step < plan.steps.len() {
            // Check cancellation signal
            if self.cancellation_mgr.is_cancelled(&task_id) {
                state_machine.transition_to(OrchestratorState::Cancelled)?;
                self.task_registry
                    .update_status(&task_id, TaskStatus::Cancelled, None);
                let _ = TaskRecoveryManager::save_checkpoint(
                    &self.db,
                    &plan,
                    OrchestratorState::Cancelled,
                );

                OrchestrationEventPublisher::publish_task_event(
                    &self.event_bus,
                    EventType::TaskCancelled,
                    &task_id,
                    serde_json::json!({ "reason": "User requested cancellation" }),
                );

                return Ok(ExecutionDecision::Fail {
                    error: "Task execution cancelled".to_string(),
                });
            }

            // Check model call limit
            if model_call_count > self.limits.max_model_calls {
                state_machine.transition_to(OrchestratorState::Blocked)?;
                self.task_registry
                    .update_status(&task_id, TaskStatus::Blocked, None);
                return Ok(ExecutionDecision::Block {
                    reason: format!(
                        "Exceeded max model call limit ({})",
                        self.limits.max_model_calls
                    ),
                });
            }

            let curr_idx = plan.current_step;
            let step = &mut plan.steps[curr_idx];

            if step.status == StepStatus::Completed || step.status == StepStatus::Skipped {
                plan.current_step += 1;
                continue;
            }

            OrchestrationEventPublisher::publish_task_event(
                &self.event_bus,
                EventType::StepStarted,
                &task_id,
                serde_json::json!({
                    "step_id": step.id,
                    "order": step.order,
                    "objective": step.objective,
                    "action_type": step.action_type.to_string()
                }),
            );

            // Phase 12 Model Selection Interface invocation
            let exec_req = ExecutionRequest {
                task_id: task_id.clone(),
                session_id: session_id.clone(),
                workspace_id: workspace_id.clone(),
                purpose: step.objective.clone(),
                context_requirements: None,
                capability_requirements: vec![],
                latency_requirement: None,
                execution_constraints: std::collections::HashMap::new(),
            };

            let target = self
                .router_interface
                .select_target(&exec_req, &self.model_registry)?;
            model_call_count += 1;

            // Execute current step
            let step_res = StepExecutor::execute_step(
                step,
                &task_id,
                session_id.as_ref(),
                workspace_id.as_ref(),
                workspace_path.as_deref(),
                &self.model_runtime,
                &target.model_id,
                &self.tool_runtime,
                &self.filesystem_security,
                &self.context_engine,
                &self.privacy_gate,
                self.verifier.as_ref(),
            )
            .await;

            match step_res {
                Ok(_output) => {
                    OrchestrationEventPublisher::publish_task_event(
                        &self.event_bus,
                        EventType::StepCompleted,
                        &task_id,
                        serde_json::json!({
                            "step_id": step.id,
                            "order": step.order,
                            "status": "COMPLETED"
                        }),
                    );

                    plan.current_step += 1;
                    let _ = TaskRecoveryManager::save_checkpoint(
                        &self.db,
                        &plan,
                        OrchestratorState::Executing,
                    );
                }
                Err(OrchestrationError::UserConsentRequired { prompt }) => {
                    state_machine.transition_to(OrchestratorState::WaitingForUser)?;
                    self.task_registry
                        .update_status(&task_id, TaskStatus::WaitingForUser, None);
                    let _ = TaskRecoveryManager::save_checkpoint(
                        &self.db,
                        &plan,
                        OrchestratorState::WaitingForUser,
                    );

                    OrchestrationEventPublisher::publish_task_event(
                        &self.event_bus,
                        EventType::TaskWaitingForUser,
                        &task_id,
                        serde_json::json!({ "prompt": prompt }),
                    );

                    return Ok(ExecutionDecision::AskUser { prompt });
                }
                Err(err) => {
                    // Evaluate retry logic
                    let retry_decision =
                        RetryManager::evaluate_retry(&err, step.attempts, &self.limits);
                    if retry_decision.should_retry {
                        state_machine.transition_to(OrchestratorState::Retrying)?;
                        self.task_registry
                            .update_status(&task_id, TaskStatus::Retrying, None);

                        OrchestrationEventPublisher::publish_task_event(
                            &self.event_bus,
                            EventType::TaskRetryStarted,
                            &task_id,
                            serde_json::json!({
                                "step_id": step.id,
                                "attempt": step.attempts,
                                "reason": retry_decision.reason
                            }),
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            retry_decision.backoff_ms,
                        ))
                        .await;
                        state_machine.transition_to(OrchestratorState::Executing)?;
                        self.task_registry
                            .update_status(&task_id, TaskStatus::Executing, None);
                        continue;
                    } else {
                        state_machine.transition_to(OrchestratorState::Failed)?;
                        self.task_registry.update_status(
                            &task_id,
                            TaskStatus::Failed,
                            Some(err.clone().into()),
                        );
                        let _ = TaskRecoveryManager::save_checkpoint(
                            &self.db,
                            &plan,
                            OrchestratorState::Failed,
                        );

                        OrchestrationEventPublisher::publish_task_event(
                            &self.event_bus,
                            EventType::TaskFailed,
                            &task_id,
                            serde_json::json!({ "error": err.to_string() }),
                        );

                        return Ok(ExecutionDecision::Fail {
                            error: err.to_string(),
                        });
                    }
                }
            }
        }

        // Verify independent task completion
        state_machine.transition_to(OrchestratorState::Verifying)?;
        self.task_registry
            .update_status(&task_id, TaskStatus::Verifying, None);

        match self.verifier.verify_completion(&plan) {
            Ok(true) => {
                // Phase 13: when a VerificationEngine is attached, the terminal
                // completion decision is routed through its CompletionGate.
                // Legacy `verify_completion == true` alone never completes.
                if let Some(engine) = self.verification_engine.clone() {
                    return self
                        .gated_complete(
                            &task_id,
                            session_id.clone(),
                            workspace_id.clone(),
                            &mut plan,
                            &mut state_machine,
                            engine,
                        )
                        .await;
                }
                state_machine.transition_to(OrchestratorState::Completed)?;
                plan.status = PlanStatus::Completed;
                self.task_registry
                    .update_status(&task_id, TaskStatus::Completed, None);
                let _ = TaskRecoveryManager::save_checkpoint(
                    &self.db,
                    &plan,
                    OrchestratorState::Completed,
                );

                OrchestrationEventPublisher::publish_task_event(
                    &self.event_bus,
                    EventType::TaskCompleted,
                    &task_id,
                    serde_json::json!({ "summary": "Task plan completed and verified successfully" }),
                );

                Ok(ExecutionDecision::Complete {
                    summary: format!("Task '{}' completed successfully", plan.objective),
                })
            }
            Ok(false) | Err(_) => {
                state_machine.transition_to(OrchestratorState::Failed)?;
                plan.status = PlanStatus::Failed;
                self.task_registry
                    .update_status(&task_id, TaskStatus::Failed, None);
                let _ = TaskRecoveryManager::save_checkpoint(
                    &self.db,
                    &plan,
                    OrchestratorState::Failed,
                );

                Ok(ExecutionDecision::Fail {
                    error: "Independent task completion verification failed".to_string(),
                })
            }
        }
    }

    /// Phase 13 completion gate: independently validate completion through the
    /// VerificationEngine before marking anything Completed.
    async fn gated_complete(
        &self,
        task_id: &TaskId,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
        plan: &mut TaskPlan,
        state_machine: &mut OrchestratorStateMachine,
        engine: Arc<VerificationEngine>,
    ) -> Result<ExecutionDecision, OrchestrationError> {
        let request =
            Self::build_verification_request(task_id, plan, workspace_id, session_id.clone());
        let result =
            engine
                .verify_task(&request)
                .map_err(|e| OrchestrationError::VerificationFailed {
                    check_name: "verification_engine".to_string(),
                    reason: e.to_string(),
                })?;

        // Supplemental AI advisory (untrusted, warnings-only, best effort).
        if request.allow_ai_advisory && !result.status.is_pass() {
            let advisory = engine
                .ai_advisory_warnings(
                    &result,
                    Some(&plan.objective),
                    session_id.as_ref().map(|s| s.as_str()),
                )
                .await;
            if !advisory.is_empty() {
                OrchestrationEventPublisher::publish_task_event(
                    &self.event_bus,
                    EventType::TaskVerificationAdvisory,
                    task_id,
                    serde_json::json!({ "advisory": advisory }),
                );
            }
        }

        match engine.completion_gate(&result, false) {
            crate::verification::CompletionDecision::AllowCompletion => {
                state_machine.transition_to(OrchestratorState::Completed)?;
                plan.status = PlanStatus::Completed;
                self.task_registry
                    .update_status(task_id, TaskStatus::Completed, None);
                let _ = TaskRecoveryManager::save_checkpoint(
                    &self.db,
                    plan,
                    OrchestratorState::Completed,
                );

                OrchestrationEventPublisher::publish_task_event(
                    &self.event_bus,
                    EventType::TaskCompleted,
                    task_id,
                    serde_json::json!({
                        "summary": "Task plan completed and independently verified",
                        "verification_id": result.verification_id,
                        "confidence": result.confidence,
                    }),
                );

                Ok(ExecutionDecision::Complete {
                    summary: format!("Task '{}' completed and verified", plan.objective),
                })
            }
            crate::verification::CompletionDecision::RequireRepair => {
                match engine.register_repair_attempt(&task_id.to_string()) {
                    Ok(attempt) => {
                        state_machine.transition_to(OrchestratorState::Retrying)?;
                        self.task_registry
                            .update_status(task_id, TaskStatus::Retrying, None);
                        let _ = TaskRecoveryManager::save_checkpoint(
                            &self.db,
                            plan,
                            OrchestratorState::Retrying,
                        );
                        Ok(ExecutionDecision::Retry {
                            reason: format!(
                                "Verification failed; bounded repair attempt {}/{}: {}",
                                attempt,
                                engine.max_repair_attempts(),
                                result.failures.join("; ")
                            ),
                            attempt,
                        })
                    }
                    Err(_) => {
                        // Repair budget exhausted: escalate to the user, never loop.
                        state_machine.transition_to(OrchestratorState::WaitingForUser)?;
                        self.task_registry
                            .update_status(task_id, TaskStatus::WaitingForUser, None);
                        Ok(ExecutionDecision::AskUser {
                            prompt: format!(
                                "Verification failed and repair budget is exhausted: {}. Please review or revise the objective.",
                                result.failures.join("; ")
                            ),
                        })
                    }
                }
            }
            crate::verification::CompletionDecision::RequireUser => {
                state_machine.transition_to(OrchestratorState::WaitingForUser)?;
                self.task_registry
                    .update_status(task_id, TaskStatus::WaitingForUser, None);
                Ok(ExecutionDecision::AskUser {
                    prompt: format!(
                        "Task execution finished but verification requires user review: {}",
                        result.failures.join("; ")
                    ),
                })
            }
            crate::verification::CompletionDecision::Blocked => {
                state_machine.transition_to(OrchestratorState::Blocked)?;
                plan.status = PlanStatus::Failed;
                self.task_registry
                    .update_status(task_id, TaskStatus::Blocked, None);
                let _ = TaskRecoveryManager::save_checkpoint(
                    &self.db,
                    plan,
                    OrchestratorState::Blocked,
                );
                Ok(ExecutionDecision::Block {
                    reason: format!(
                        "Completion blocked by verification: {}",
                        result.failures.join("; ")
                    ),
                })
            }
        }
    }

    /// Build an authoritative verification request from the executed plan.
    /// Only workspace-relative, traversal-free file probes are included; step
    /// outcomes come from the orchestrator's own step store (never model claims).
    fn build_verification_request(
        task_id: &TaskId,
        plan: &TaskPlan,
        workspace_id: Option<WorkspaceId>,
        session_id: Option<SessionId>,
    ) -> TaskVerificationRequest {
        let mut request = TaskVerificationRequest::for_task(task_id.to_string());
        request.workspace_id = workspace_id.map(|w| w.to_string());
        request.session_id = session_id.map(|s| s.to_string());

        for step in &plan.steps {
            request.steps.push(StepEvidenceInput {
                step_id: step.id.clone(),
                status: step.status.to_string(),
                has_error: step.status == StepStatus::Failed,
            });

            if step.action_type == StepActionType::WriteFile {
                for key in ["rel_path", "path"] {
                    if let Some(rel) = step
                        .inputs
                        .as_ref()
                        .and_then(|i| i.get(key))
                        .and_then(|v| v.as_str())
                    {
                        if !rel.trim().is_empty()
                            && !rel.starts_with('/')
                            && !rel.starts_with('\\')
                            && !rel.contains("..")
                        {
                            request.expected_files.push(ExpectedFile::exists(rel));
                        }
                    }
                }
            }
        }

        for constraint in &plan.constraints {
            request
                .constraints
                .push(Self::map_plan_constraint(constraint));
        }

        request
    }

    fn map_plan_constraint(raw: &str) -> TaskConstraint {
        let lower = raw.to_lowercase();
        let constraint_type = if lower.contains("local") && !lower.contains("online") {
            "local_only"
        } else if lower.contains("cloud") || lower.contains("online") {
            "no_cloud"
        } else if lower.contains("workspace") || lower.contains("modif") || lower.contains("source")
        {
            "workspace_only"
        } else if lower.contains("format")
            || lower.contains("excel")
            || lower.contains("pdf")
            || lower.contains("csv")
        {
            "output_format"
        } else {
            "custom"
        };
        TaskConstraint {
            constraint_type: constraint_type.to_string(),
            description: raw.to_string(),
            observed: None,
        }
    }
}
