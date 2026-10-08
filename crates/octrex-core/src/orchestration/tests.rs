#[cfg(test)]
mod tests {
    use crate::app::ApplicationState;
    use crate::ids::TaskId;
    use crate::orchestration::{
        limits::OrchestratorLimits,
        plan::PlanValidator,
        planner::TaskPlanner,
        state::OrchestratorStateMachine,
        types::{
            ExecutionDecision, OrchestratorState, PlanStatus, StepActionType, StepStatus, TaskPlan,
            TaskStep,
        },
        OrchestrationError, OrchestrationService,
    };
    use std::sync::Arc;

    #[test]
    fn test_orchestrator_state_transitions() {
        let mut sm = OrchestratorStateMachine::new();
        assert_eq!(sm.state(), OrchestratorState::Created);

        // Created -> Planning
        assert!(sm.transition_to(OrchestratorState::Planning).is_ok());
        assert_eq!(sm.state(), OrchestratorState::Planning);

        // Planning -> PlanReady
        assert!(sm.transition_to(OrchestratorState::PlanReady).is_ok());
        assert_eq!(sm.state(), OrchestratorState::PlanReady);

        // PlanReady -> Executing
        assert!(sm.transition_to(OrchestratorState::Executing).is_ok());
        assert_eq!(sm.state(), OrchestratorState::Executing);

        // Executing -> WaitingForUser
        assert!(sm.transition_to(OrchestratorState::WaitingForUser).is_ok());

        // WaitingForUser -> Executing
        assert!(sm.transition_to(OrchestratorState::Executing).is_ok());

        // Executing -> Completed
        assert!(sm.transition_to(OrchestratorState::Completed).is_ok());

        // Invalid: Completed -> Executing
        assert!(sm.transition_to(OrchestratorState::Executing).is_err());
    }

    #[test]
    fn test_task_plan_creation_and_validation() {
        let task_id = TaskId::new();
        let limits = OrchestratorLimits::default();
        let plan = TaskPlanner::create_plan(
            task_id.clone(),
            "Inspect workspace files and generate report",
            &limits,
            None,
        )
        .unwrap();

        assert_eq!(plan.task_id, task_id);
        assert!(!plan.steps.is_empty());
        assert_eq!(plan.status, PlanStatus::Approved);

        // Validate plan
        assert!(PlanValidator::validate(&plan, &limits, None).is_ok());
    }

    #[test]
    fn test_cyclic_dependency_rejection() {
        let task_id = TaskId::new();
        let limits = OrchestratorLimits::default();
        let mut plan = TaskPlan::new(task_id, "Test cyclic task");

        let mut step1 = TaskStep::new(1, "Step 1", StepActionType::ThinkAnalyze);
        step1.id = "step-1".to_string();
        step1.dependencies = vec!["step-2".to_string()];

        let mut step2 = TaskStep::new(2, "Step 2", StepActionType::ModelCall);
        step2.id = "step-2".to_string();
        step2.dependencies = vec!["step-1".to_string()];

        plan.steps.push(step1);
        plan.steps.push(step2);

        let err = PlanValidator::validate(&plan, &limits, None).unwrap_err();
        assert!(matches!(err, OrchestrationError::InvalidPlan { .. }));
    }

    #[tokio::test]
    async fn test_orchestration_service_full_workflow() {
        let app_state = ApplicationState::initialize();

        // Fail-closed test setup: register an explicit mock provider + model.
        // The workflow's ModelCall step cannot (and must not) succeed with an
        // empty registry or an ambient cloud fallback.
        let mock_provider = Arc::new(crate::providers::MockAdapter::new(
            "test_provider",
            crate::providers::ExecutionMode::Local,
        ));
        app_state.provider_registry.register_provider(mock_provider);
        app_state
            .model_registry
            .register(crate::models::ModelDescriptor {
                id: "test-model".to_string(),
                provider_id: "test_provider".to_string(),
                model_identifier: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                execution_mode: crate::providers::ExecutionMode::Local,
                capabilities: vec![],
                context_window: Some(8192),
                max_output_tokens: Some(2048),
                tokenizer: crate::models::TokenizerInfo::Estimated { factor: 4.0 },
                hardware_requirements: None,
                availability: crate::models::ModelAvailability::Available,
                metadata: std::collections::HashMap::new(),
            });

        let orchestration = OrchestrationService::new(
            OrchestratorLimits::default(),
            app_state.task_registry.clone(),
            app_state.db.clone(),
            app_state.event_bus.clone(),
            app_state.model_runtime.clone(),
            app_state.model_registry.clone(),
            app_state.tool_runtime.clone(),
            app_state.filesystem_security.clone(),
            app_state.context_engine.clone(),
            app_state.privacy_gate.clone(),
        );

        let temp = std::env::temp_dir();
        let ws_path = temp.to_str().unwrap().to_string();
        let readme_file = temp.join("README.md");
        let _ = std::fs::write(&readme_file, "Octrex Test Workspace README");

        let (task, plan) = orchestration
            .create_task_and_plan("Read and analyze files", None, None)
            .unwrap();

        assert_eq!(task.title, "Read and analyze files");
        assert!(!plan.steps.is_empty());

        let res = orchestration
            .start_task(&task.id, Some(ws_path))
            .await
            .unwrap();
        assert!(matches!(res, ExecutionDecision::Complete { .. }));

        let updated_task = app_state.task_registry.get_task(&task.id).unwrap();
        assert_eq!(updated_task.status, crate::task::TaskStatus::Completed);
    }
}
