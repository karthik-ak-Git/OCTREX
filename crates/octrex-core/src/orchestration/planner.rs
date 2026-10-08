use super::errors::OrchestrationError;
use super::limits::OrchestratorLimits;
use super::plan::PlanValidator;
use super::types::{PlanStatus, StepActionType, TaskPlan, TaskStep};
use crate::ids::TaskId;
use crate::tools::ToolRegistry;

pub struct TaskPlanner;

impl TaskPlanner {
    /// Create a deterministic structured plan for a given task objective
    pub fn create_plan(
        task_id: TaskId,
        objective: &str,
        limits: &OrchestratorLimits,
        tool_registry: Option<&ToolRegistry>,
    ) -> Result<TaskPlan, OrchestrationError> {
        let mut plan = TaskPlan::new(task_id, objective);

        let lower_obj = objective.to_lowercase();

        // Build logical steps based on task analysis
        let mut order = 1;

        // Step 1: Initial Context & Objective Analysis
        let analyze_step = TaskStep::new(
            order,
            "Analyze objective and retrieve workspace context",
            StepActionType::ThinkAnalyze,
        );
        let analyze_id = analyze_step.id.clone();
        plan.steps.push(analyze_step);
        order += 1;

        // Step 2: Context Retrieval
        let retrieve_step = TaskStep::new(
            order,
            "Retrieve workspace context and related files",
            StepActionType::RetrieveContext,
        )
        .with_dependencies(vec![analyze_id.clone()]);
        let retrieve_id = retrieve_step.id.clone();
        plan.steps.push(retrieve_step);
        order += 1;

        if lower_obj.contains("read")
            || lower_obj.contains("inspect")
            || lower_obj.contains("file")
            || lower_obj.contains("report")
            || lower_obj.contains("analyze")
        {
            let read_step = TaskStep::new(
                order,
                "Inspect relevant workspace files",
                StepActionType::ReadFile,
            )
            .with_dependencies(vec![retrieve_id.clone()]);
            let read_id = read_step.id.clone();
            plan.steps.push(read_step);
            order += 1;

            let model_step = TaskStep::new(
                order,
                "Perform model synthesis and analysis",
                StepActionType::ModelCall,
            )
            .with_dependencies(vec![read_id.clone()]);
            let model_id = model_step.id.clone();
            plan.steps.push(model_step);
            order += 1;

            if lower_obj.contains("write")
                || lower_obj.contains("create")
                || lower_obj.contains("report")
                || lower_obj.contains("generate")
            {
                let write_step = TaskStep::new(
                    order,
                    "Write output artifact to workspace",
                    StepActionType::WriteFile,
                )
                .with_dependencies(vec![model_id.clone()]);
                let write_id = write_step.id.clone();
                plan.steps.push(write_step);
                order += 1;

                let verify_step = TaskStep::new(
                    order,
                    "Verify generated output file",
                    StepActionType::Verify,
                )
                .with_dependencies(vec![write_id.clone()])
                .with_verification(true);
                let verify_id = verify_step.id.clone();
                plan.steps.push(verify_step);
                order += 1;

                let complete_step =
                    TaskStep::new(order, "Finalize task completion", StepActionType::Complete)
                        .with_dependencies(vec![verify_id]);
                plan.steps.push(complete_step);
            } else {
                let verify_step = TaskStep::new(
                    order,
                    "Verify task analysis results",
                    StepActionType::Verify,
                )
                .with_dependencies(vec![model_id.clone()])
                .with_verification(true);
                let verify_id = verify_step.id.clone();
                plan.steps.push(verify_step);
                order += 1;

                let complete_step =
                    TaskStep::new(order, "Finalize task completion", StepActionType::Complete)
                        .with_dependencies(vec![verify_id]);
                plan.steps.push(complete_step);
            }
        } else {
            let model_step = TaskStep::new(
                order,
                "Execute core task model processing",
                StepActionType::ModelCall,
            )
            .with_dependencies(vec![retrieve_id.clone()]);
            let model_id = model_step.id.clone();
            plan.steps.push(model_step);
            order += 1;

            let verify_step = TaskStep::new(
                order,
                "Verify model outputs and criteria",
                StepActionType::Verify,
            )
            .with_dependencies(vec![model_id.clone()])
            .with_verification(true);
            let verify_id = verify_step.id.clone();
            plan.steps.push(verify_step);
            order += 1;

            let complete_step =
                TaskStep::new(order, "Finalize task completion", StepActionType::Complete)
                    .with_dependencies(vec![verify_id]);
            plan.steps.push(complete_step);
        }

        // Validate generated plan
        PlanValidator::validate(&plan, limits, tool_registry)?;
        plan.status = PlanStatus::Approved;

        Ok(plan)
    }

    /// Parse a plan proposed by a model response and validate it strictly
    pub fn parse_model_proposed_plan(
        task_id: TaskId,
        objective: &str,
        raw_proposal: &str,
        limits: &OrchestratorLimits,
        tool_registry: Option<&ToolRegistry>,
    ) -> Result<TaskPlan, OrchestrationError> {
        let parsed_json: serde_json::Value =
            serde_json::from_str(raw_proposal).map_err(|e| OrchestrationError::InvalidPlan {
                reason: format!("Model plan proposal is not valid JSON: {}", e),
            })?;

        let steps_arr = parsed_json
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| OrchestrationError::InvalidPlan {
                reason: "Model plan proposal missing 'steps' array".to_string(),
            })?;

        let mut plan = TaskPlan::new(task_id, objective);
        let mut order = 1;

        for (idx, step_val) in steps_arr.iter().enumerate() {
            let obj = step_val
                .get("objective")
                .and_then(|v| v.as_str())
                .unwrap_or("Execute step");
            let action_str = step_val
                .get("action_type")
                .and_then(|v| v.as_str())
                .unwrap_or("MODEL_CALL");

            let action_type = match action_str.to_uppercase().as_str() {
                "THINK_ANALYZE" | "THINKANALYZE" => StepActionType::ThinkAnalyze,
                "RETRIEVE_CONTEXT" | "RETRIEVECONTEXT" => StepActionType::RetrieveContext,
                "READ_FILE" | "READFILE" => StepActionType::ReadFile,
                "WRITE_FILE" | "WRITEFILE" => StepActionType::WriteFile,
                "EXECUTE_TOOL" | "EXECUTETOOL" => StepActionType::ExecuteTool,
                "VERIFY" => StepActionType::Verify,
                "ASK_USER" | "ASKUSER" => StepActionType::AskUser,
                "COMPLETE" => StepActionType::Complete,
                _ => StepActionType::ModelCall,
            };

            let mut step = TaskStep::new(order, obj, action_type);
            step.id = format!("step-{}", idx + 1);

            if let Some(deps_arr) = step_val.get("dependencies").and_then(|v| v.as_array()) {
                step.dependencies = deps_arr
                    .iter()
                    .filter_map(|d| d.as_str().map(|s| s.to_string()))
                    .collect();
            }

            if let Some(req_user) = step_val
                .get("requires_user_input")
                .and_then(|v| v.as_bool())
            {
                step.requires_user_input = req_user;
            }

            if let Some(inputs) = step_val.get("inputs") {
                step.inputs = Some(inputs.clone());
            }

            plan.steps.push(step);
            order += 1;
        }

        PlanValidator::validate(&plan, limits, tool_registry)?;
        plan.status = PlanStatus::Approved;

        Ok(plan)
    }
}
