use crate::context::budget::TokenBudget;
use crate::context::errors::ContextError;
use crate::context::item::ContextItem;
use crate::context::types::{ContextRole, TokenCountKind};
use crate::privacy::PrivacyClassification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledContext {
    pub model_id: String,
    pub context_window: usize,
    pub input_budget: usize,
    pub reserved_output: usize,
    pub safety_margin: usize,
    pub items: Vec<ContextItem>,
    pub total_tokens: usize,
    pub token_count_kind: TokenCountKind,
    pub compaction_performed: bool,
    pub compaction_events: Vec<String>,
    pub provenance_summary: Vec<String>,
    pub aggregate_classification: PrivacyClassification,
}

pub struct ContextAssembler;

impl ContextAssembler {
    /// Assembles selected context items into a structured request context
    /// Enforces ROLE SAFETY invariants:
    ///   - FileContent, ToolResult, ModelOutput, CompactionSummary CANNOT assume System role.
    pub fn assemble(
        items: Vec<ContextItem>,
        budget: &TokenBudget,
        model_id: &str,
        compaction_performed: bool,
        compaction_events: Vec<String>,
    ) -> Result<AssembledContext, ContextError> {
        let mut total_tokens = 0;
        let mut aggregate_classification = PrivacyClassification::Public;
        let mut aggregate_kind = TokenCountKind::Exact;
        let mut provenance_summary = Vec::new();

        for item in &items {
            // ROLE SAFETY CHECK
            if item.role == ContextRole::System && item.source.is_data_plane() {
                return Err(ContextError::InvalidContextRole {
                    source_name: item.source.as_str().to_string(),
                    role: "system".to_string(),
                });
            }

            let tokens = item.token_count.unwrap_or(0);
            total_tokens += tokens;

            // Classification propagation: aggregate = max(item classifications)
            if item.classification > aggregate_classification {
                aggregate_classification = item.classification;
            }

            if item.token_count_kind == TokenCountKind::Unknown {
                aggregate_kind = TokenCountKind::Unknown;
            } else if item.token_count_kind == TokenCountKind::Estimated
                && aggregate_kind == TokenCountKind::Exact
            {
                aggregate_kind = TokenCountKind::Estimated;
            }

            provenance_summary.push(item.safe_summary());
        }

        if total_tokens > budget.usable_input_budget {
            return Err(ContextError::BudgetExceeded {
                needed: total_tokens,
                available: budget.usable_input_budget,
            });
        }

        Ok(AssembledContext {
            model_id: model_id.to_string(),
            context_window: budget.context_window,
            input_budget: budget.usable_input_budget,
            reserved_output: budget.reserved_output,
            safety_margin: budget.safety_margin,
            items,
            total_tokens,
            token_count_kind: aggregate_kind,
            compaction_performed,
            compaction_events,
            provenance_summary,
            aggregate_classification,
        })
    }
}
