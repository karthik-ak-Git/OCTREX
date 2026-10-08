use crate::context::assembler::{AssembledContext, ContextAssembler};
use crate::context::budget::{BudgetPolicy, TokenBudget};
use crate::context::checkpoint::{BudgetSnapshot, ContextCheckpoint};
use crate::context::compaction::{CompactionEngine, CompactionResult};
use crate::context::errors::ContextError;
use crate::context::item::ContextItem;
use crate::context::policy::ContextSecurityPolicy;
use crate::context::selector::ContextSelector;
use crate::context::state::TaskContextState;
use crate::context::tokenizer::ContextTokenCounter;
use crate::context::types::{ContextRole, ContextSource};
use crate::db::DatabaseManager;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::privacy::PrivacyClassification;
use crate::tools::response::ToolResponse;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;

/// Central production ContextEngine Service
pub struct ContextService {
    event_bus: Arc<EventBus>,
    // NOTE (hardening pass): `db` is currently unused — session items live in
    // the in-memory cache and are not yet persisted to `context_items` /
    // `context_checkpoints` tables (migration v6). Kept intentionally so the
    // persistence wiring can land without an API break; tracked as tech debt.
    #[allow(dead_code)]
    db: Arc<DatabaseManager>,
    policy: BudgetPolicy,
    counter: ContextTokenCounter,
    compactor: CompactionEngine,
    // Session context item storage (in-memory cache backed by DB)
    session_items: RwLock<HashMap<String, Vec<ContextItem>>>,
    checkpoints: RwLock<HashMap<String, Vec<ContextCheckpoint>>>,
    task_states: RwLock<HashMap<String, TaskContextState>>,
}

impl ContextService {
    pub fn new(event_bus: Arc<EventBus>, db: Arc<DatabaseManager>) -> Self {
        Self {
            event_bus,
            db,
            policy: BudgetPolicy::default(),
            counter: ContextTokenCounter::default_counter(),
            compactor: CompactionEngine::default(),
            session_items: RwLock::new(HashMap::new()),
            checkpoints: RwLock::new(HashMap::new()),
            task_states: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_status(&self) -> serde_json::Value {
        let items_guard = self.session_items.read().unwrap();
        let checkpoints_guard = self.checkpoints.read().unwrap();

        let total_items: usize = items_guard.values().map(|v| v.len()).sum();
        let total_checkpoints: usize = checkpoints_guard.values().map(|v| v.len()).sum();

        serde_json::json!({
            "service": "ContextEngineService",
            "status": "READY",
            "active_sessions": items_guard.len(),
            "total_items_tracked": total_items,
            "total_checkpoints": total_checkpoints,
            "fail_closed": true,
            "compaction_engine": "bounded_deterministic",
        })
    }

    pub fn add_item(&self, item: ContextItem) -> Result<ContextItem, ContextError> {
        ContextSecurityPolicy::validate_item(&item)?;

        let session_key = item
            .session_id
            .as_ref()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "global".to_string());

        let mut item_with_tokens = item;
        if item_with_tokens.token_count.is_none() {
            let (count, kind) = self.counter.count(&item_with_tokens.content)?;
            item_with_tokens.token_count = Some(count);
            item_with_tokens.token_count_kind = kind;
        }

        {
            let mut guard = self.session_items.write().unwrap();
            guard
                .entry(session_key.clone())
                .or_insert_with(Vec::new)
                .push(item_with_tokens.clone());
        }

        // Emit ContextItemAdded event
        let _ = self.event_bus.publish(EventEnvelope::new(
            EventType::ContextItemAdded,
            serde_json::json!({
                "item_id": item_with_tokens.id.as_str(),
                "source": item_with_tokens.source.as_str(),
                "role": item_with_tokens.role.to_model_role(),
                "classification": item_with_tokens.classification.to_string(),
                "session_id": session_key,
                "token_count": item_with_tokens.token_count
            }),
        ));

        Ok(item_with_tokens)
    }

    pub fn ingest_tool_result(
        &self,
        resp: &ToolResponse,
        classification: PrivacyClassification,
        workspace_id: Option<WorkspaceId>,
        session_id: Option<SessionId>,
        task_id: Option<TaskId>,
    ) -> ContextItem {
        let content = serde_json::to_string(&resp.result).unwrap_or_default();
        let mut item =
            ContextItem::new(ContextSource::ToolResult, ContextRole::ToolResult, content)
                .with_source_id(resp.tool_id.as_str())
                .with_classification(classification)
                .with_workspace(workspace_id)
                .with_session(session_id)
                .with_task(task_id)
                .with_provenance(format!("tool_execution:{}", resp.tool_call_id));

        item.priority = 40;
        let _ = self.add_item(item.clone());
        item
    }

    pub fn ingest_model_output(
        &self,
        content: &str,
        model_id: &str,
        classification: PrivacyClassification,
        session_id: Option<SessionId>,
        task_id: Option<TaskId>,
    ) -> ContextItem {
        let mut item = ContextItem::new(
            ContextSource::ModelOutput,
            ContextRole::ModelOutput,
            content,
        )
        .with_source_id(model_id)
        .with_classification(classification)
        .with_session(session_id)
        .with_task(task_id)
        .with_provenance(format!("model_generation:{}", model_id));

        item.priority = 35;
        let _ = self.add_item(item.clone());
        item
    }

    pub fn get_budget_for_session(
        &self,
        session_id: &str,
        model_id: &str,
        context_window: usize,
    ) -> Result<TokenBudget, ContextError> {
        let mut budget = TokenBudget::compute(model_id, context_window, None, &self.policy)?;

        let guard = self.session_items.read().unwrap();
        if let Some(items) = guard.get(session_id) {
            let used: usize = items.iter().map(|i| i.token_count.unwrap_or(0)).sum();
            budget.current_usage = used;
        }

        Ok(budget)
    }

    pub fn get_session_context_summary(
        &self,
        session_id: &str,
    ) -> Result<serde_json::Value, ContextError> {
        let guard = self.session_items.read().unwrap();
        let items = guard.get(session_id).cloned().unwrap_or_default();

        let total_tokens: usize = items.iter().map(|i| i.token_count.unwrap_or(0)).sum();
        let safe_summaries: Vec<String> = items.iter().map(|i| i.safe_summary()).collect();

        Ok(serde_json::json!({
            "session_id": session_id,
            "total_items": items.len(),
            "total_tokens": total_tokens,
            "safe_summaries": safe_summaries
        }))
    }

    pub fn preview_context(
        &self,
        session_id: &str,
        task_id: Option<&str>,
        model_id: &str,
        context_window: usize,
        user_request: Option<&str>,
    ) -> Result<AssembledContext, ContextError> {
        info!(
            session_id,
            model_id, "Executing local diagnostic context preview"
        );

        let _ = self.event_bus.publish(EventEnvelope::new(
            EventType::ContextBuildStarted,
            serde_json::json!({
                "session_id": session_id,
                "task_id": task_id,
                "model_id": model_id
            }),
        ));

        let budget = TokenBudget::compute(model_id, context_window, None, &self.policy)?;

        let mut items = {
            let guard = self.session_items.read().unwrap();
            guard.get(session_id).cloned().unwrap_or_default()
        };

        if let Some(req_text) = user_request {
            let mut user_item =
                ContextItem::new(ContextSource::UserRequest, ContextRole::User, req_text);
            user_item.priority = 10;
            let (count, kind) = self.counter.count(req_text)?;
            user_item.token_count = Some(count);
            user_item.token_count_kind = kind;
            items.push(user_item);
        }

        let selection = ContextSelector::select(items, &budget, &self.counter)?;

        let assembled =
            ContextAssembler::assemble(selection.selected, &budget, model_id, false, Vec::new())?;

        let _ = self.event_bus.publish(EventEnvelope::new(
            EventType::ContextBuildCompleted,
            serde_json::json!({
                "session_id": session_id,
                "task_id": task_id,
                "model_id": model_id,
                "total_tokens": assembled.total_tokens,
                "items_selected": assembled.items.len(),
                "aggregate_classification": assembled.aggregate_classification.to_string()
            }),
        ));

        Ok(assembled)
    }

    pub fn compact_context(
        &self,
        session_id: &str,
        _task_id: Option<&str>,
        model_id: &str,
        context_window: usize,
    ) -> Result<CompactionResult, ContextError> {
        let budget = TokenBudget::compute(model_id, context_window, None, &self.policy)?;

        let items = {
            let guard = self.session_items.read().unwrap();
            guard.get(session_id).cloned().unwrap_or_default()
        };

        let result = self.compactor.compact(items, &budget, &self.counter)?;

        // Update in-memory session items with compacted list
        {
            let mut guard = self.session_items.write().unwrap();
            guard.insert(session_id.to_string(), result.compacted_items.clone());
        }

        let _ = self.event_bus.publish(EventEnvelope::new(
            EventType::ContextCompactionCompleted,
            serde_json::json!({
                "session_id": session_id,
                "rounds": result.rounds,
                "tokens_before": result.tokens_before,
                "tokens_after": result.tokens_after,
                "items_compacted": result.items_compacted
            }),
        ));

        Ok(result)
    }

    pub fn create_checkpoint(
        &self,
        session_id: &str,
        task_id: Option<&str>,
        model_id: &str,
        assembled: &AssembledContext,
    ) -> Result<ContextCheckpoint, ContextError> {
        let budget_snapshot = BudgetSnapshot {
            context_window: assembled.context_window,
            reserved_output: assembled.reserved_output,
            safety_margin: assembled.safety_margin,
            usable_input_budget: assembled.input_budget,
            total_used: assembled.total_tokens,
        };

        let mut chk = ContextCheckpoint::new(
            session_id,
            task_id.map(|t| t.to_string()),
            model_id,
            budget_snapshot,
        );
        chk.selected_item_ids = assembled
            .items
            .iter()
            .map(|i| i.id.as_str().to_string())
            .collect();

        {
            let mut guard = self.checkpoints.write().unwrap();
            guard
                .entry(session_id.to_string())
                .or_insert_with(Vec::new)
                .push(chk.clone());
        }

        let _ = self.event_bus.publish(EventEnvelope::new(
            EventType::ContextCheckpointCreated,
            serde_json::json!({
                "checkpoint_id": chk.id,
                "session_id": session_id,
                "task_id": task_id,
                "model_id": model_id
            }),
        ));

        Ok(chk)
    }

    pub fn list_checkpoints(
        &self,
        session_id: &str,
    ) -> Result<Vec<ContextCheckpoint>, ContextError> {
        let guard = self.checkpoints.read().unwrap();
        Ok(guard.get(session_id).cloned().unwrap_or_default())
    }

    pub fn get_task_context_state(&self, task_id: &str) -> Result<TaskContextState, ContextError> {
        let guard = self.task_states.read().unwrap();
        Ok(guard
            .get(task_id)
            .cloned()
            .unwrap_or_else(|| TaskContextState::new(task_id, "Task objective")))
    }

    pub fn update_task_state(&self, task_id: &str, state: TaskContextState) {
        let mut guard = self.task_states.write().unwrap();
        guard.insert(task_id.to_string(), state);
    }
}
