use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSnapshot {
    pub context_window: usize,
    pub reserved_output: usize,
    pub safety_margin: usize,
    pub usable_input_budget: usize,
    pub total_used: usize,
}

/// Lightweight durable context checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCheckpoint {
    pub id: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub model_id: String,
    pub context_version: u64,
    pub selected_item_ids: Vec<String>,
    pub summary_ids: Vec<String>,
    pub task_state_version: u64,
    pub token_budget_snapshot: BudgetSnapshot,
    pub created_at: u64,
}

impl ContextCheckpoint {
    pub fn new(
        session_id: impl Into<String>,
        task_id: Option<String>,
        model_id: impl Into<String>,
        budget_snapshot: BudgetSnapshot,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: format!("chk-{}", uuid::Uuid::new_v4().simple()),
            session_id: session_id.into(),
            task_id,
            model_id: model_id.into(),
            context_version: 1,
            selected_item_ids: Vec::new(),
            summary_ids: Vec::new(),
            task_state_version: 1,
            token_budget_snapshot: budget_snapshot,
            created_at: now,
        }
    }
}
