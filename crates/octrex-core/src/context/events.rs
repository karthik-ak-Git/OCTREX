use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuildStartedPayload {
    pub session_id: String,
    pub task_id: Option<String>,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuildCompletedPayload {
    pub session_id: String,
    pub task_id: Option<String>,
    pub model_id: String,
    pub total_tokens: usize,
    pub items_selected: usize,
    pub items_excluded: usize,
    pub compaction_performed: bool,
    pub aggregate_classification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItemAddedPayload {
    pub item_id: String,
    pub source: String,
    pub role: String,
    pub classification: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCompactionPayload {
    pub session_id: String,
    pub rounds: u32,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub items_compacted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudgetExceededPayload {
    pub session_id: String,
    pub model_id: String,
    pub tokens_needed: usize,
    pub budget_available: usize,
}
