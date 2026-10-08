use super::errors::OrchestrationError;
use super::limits::OrchestratorLimits;
use super::types::{StepActionType, TaskPlan};
use crate::tools::ToolRegistry;
use std::collections::HashSet;

pub struct PlanValidator;

impl PlanValidator {
    pub fn validate(
        plan: &TaskPlan,
        limits: &OrchestratorLimits,
        tool_registry: Option<&ToolRegistry>,
    ) -> Result<(), OrchestrationError> {
        // 1. Step count check
        if plan.steps.is_empty() {
            return Err(OrchestrationError::InvalidPlan {
                reason: "Task plan must contain at least one step".to_string(),
            });
        }

        if plan.steps.len() > limits.max_steps as usize {
            return Err(OrchestrationError::LimitExceeded {
                limit_type: "max_steps".to_string(),
                limit_value: limits.max_steps as u64,
                actual_value: plan.steps.len() as u64,
            });
        }

        let mut step_ids = HashSet::new();

        // 2. Step ID uniqueness & valid action checks
        for step in &plan.steps {
            if step_ids.contains(&step.id) {
                return Err(OrchestrationError::InvalidPlan {
                    reason: format!("Duplicate step ID found in plan: '{}'", step.id),
                });
            }
            step_ids.insert(step.id.clone());

            // Check step attempt limit
            if step.max_attempts > limits.max_retries_per_step {
                return Err(OrchestrationError::InvalidPlan {
                    reason: format!(
                        "Step '{}' max_attempts ({}) exceeds limit ({})",
                        step.id, step.max_attempts, limits.max_retries_per_step
                    ),
                });
            }

            // If action is ExecuteTool, verify tool exists in registry
            if step.action_type == StepActionType::ExecuteTool {
                if let Some(registry) = tool_registry {
                    if let Some(inputs) = &step.inputs {
                        if let Some(tool_name) = inputs.get("tool_name").and_then(|v| v.as_str()) {
                            let tool_id = crate::tools::ToolId::new(tool_name);
                            if registry.get(&tool_id).is_none() {
                                return Err(OrchestrationError::InvalidPlan {
                                    reason: format!(
                                        "Plan references unregistered tool '{}'",
                                        tool_name
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 3. Check dependency graph validity (no unknown dependencies, no cycles)
        for step in &plan.steps {
            for dep in &step.dependencies {
                if !step_ids.contains(dep) {
                    return Err(OrchestrationError::InvalidPlan {
                        reason: format!(
                            "Step '{}' references non-existent dependency '{}'",
                            step.id, dep
                        ),
                    });
                }
                if dep == &step.id {
                    return Err(OrchestrationError::InvalidPlan {
                        reason: format!("Step '{}' cannot depend on itself", step.id),
                    });
                }
            }
        }

        // Cycle detection in dependencies using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn dfs(
            step_id: &str,
            plan: &TaskPlan,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
        ) -> bool {
            visited.insert(step_id.to_string());
            rec_stack.insert(step_id.to_string());

            if let Some(step) = plan.steps.iter().find(|s| s.id == step_id) {
                for dep in &step.dependencies {
                    if !visited.contains(dep) {
                        if dfs(dep, plan, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(dep) {
                        return true;
                    }
                }
            }

            rec_stack.remove(step_id);
            false
        }

        for step in &plan.steps {
            if !visited.contains(&step.id) {
                if dfs(&step.id, plan, &mut visited, &mut rec_stack) {
                    return Err(OrchestrationError::InvalidPlan {
                        reason: format!("Cyclic dependency detected involving step '{}'", step.id),
                    });
                }
            }
        }

        Ok(())
    }
}
