use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Durable representation of task execution state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskContextState {
    pub task_id: String,
    pub objective: String,
    pub requirements: Vec<String>,
    pub constraints: Vec<String>,
    pub completed_steps: Vec<String>,
    pub pending_steps: Vec<String>,
    pub blocked_steps: Vec<String>,
    pub decisions: HashMap<String, String>,
    pub artifacts: Vec<String>,
    pub verification_status: String,
    pub unresolved_questions: Vec<String>,
    pub context_checkpoint_id: Option<String>,
    pub last_model_call: Option<u64>,
    pub version: u64,
}

impl TaskContextState {
    pub fn new(task_id: impl Into<String>, objective: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            objective: objective.into(),
            requirements: Vec::new(),
            constraints: Vec::new(),
            completed_steps: Vec::new(),
            pending_steps: Vec::new(),
            blocked_steps: Vec::new(),
            decisions: HashMap::new(),
            artifacts: Vec::new(),
            verification_status: "UNVERIFIED".to_string(),
            unresolved_questions: Vec::new(),
            context_checkpoint_id: None,
            last_model_call: None,
            version: 1,
        }
    }
}
