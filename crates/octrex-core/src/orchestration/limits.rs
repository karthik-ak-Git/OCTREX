use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorLimits {
    pub max_task_duration_secs: u64,
    pub max_steps: u32,
    pub max_model_calls: u32,
    pub max_tool_calls: u32,
    pub max_retries_per_step: u32,
    pub max_recursive_planning_depth: u32,
    pub max_concurrent_actions: u32,
    pub max_context_compaction_rounds: u32,
}

impl Default for OrchestratorLimits {
    fn default() -> Self {
        Self {
            max_task_duration_secs: 600, // 10 minutes
            max_steps: 30,
            max_model_calls: 20,
            max_tool_calls: 50,
            max_retries_per_step: 3,
            max_recursive_planning_depth: 3,
            max_concurrent_actions: 1, // Sequential by default
            max_context_compaction_rounds: 5,
        }
    }
}

impl OrchestratorLimits {
    pub fn strict() -> Self {
        Self {
            max_task_duration_secs: 180,
            max_steps: 10,
            max_model_calls: 5,
            max_tool_calls: 10,
            max_retries_per_step: 1,
            max_recursive_planning_depth: 1,
            max_concurrent_actions: 1,
            max_context_compaction_rounds: 2,
        }
    }
}
