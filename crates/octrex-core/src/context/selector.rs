use crate::context::budget::TokenBudget;
use crate::context::errors::ContextError;
use crate::context::item::{ContextItem, ContextItemId};
use crate::context::tokenizer::ContextTokenCounter;
use crate::context::types::{ContextInclusionReason, TokenCountKind};

pub struct SelectionResult {
    pub selected: Vec<ContextItem>,
    pub excluded: Vec<(ContextItemId, String)>,
    pub total_tokens: usize,
    pub token_count_kind: TokenCountKind,
}

pub struct ContextSelector;

impl ContextSelector {
    /// Priority-driven deterministic context selection algorithm
    /// Priority Groups:
    ///   P0: Mandatory (System policy, Company policy, Security policy, Privacy policy, Runtime instructions)
    ///   P1: High priority (Current user request, Active task state, Verification constraints)
    ///   P2: Medium priority (Recent conversation messages, Direct tool results, Direct selected files)
    ///   P3: Secondary (Older messages, Secondary workspace context, Artifact references)
    ///   P4: Optional (Historical material, low-relevance artifacts)
    pub fn select(
        mut items: Vec<ContextItem>,
        budget: &TokenBudget,
        counter: &ContextTokenCounter,
    ) -> Result<SelectionResult, ContextError> {
        let mut selected = Vec::new();
        let mut excluded = Vec::new();
        let mut used_tokens = 0;
        let mut aggregate_kind = TokenCountKind::Exact;

        // Step 1: Filter out expired items
        items.retain(|item| {
            if item.is_expired() {
                excluded.push((item.id.clone(), "expired".to_string()));
                false
            } else {
                true
            }
        });

        // Step 2: Compute/populate token counts for items that don't have them
        for item in &mut items {
            if item.token_count.is_none() {
                let (count, kind) = counter.count(&item.content)?;
                item.token_count = Some(count);
                item.token_count_kind = kind;
            }
            aggregate_kind =
                ContextTokenCounter::merge_kinds(aggregate_kind, item.token_count_kind);
        }

        // Step 3: Sort items deterministically:
        // Primary sort: Priority asc (0 = P0, 25 = P1, 50 = P2, 75 = P3, 100 = P4)
        // Secondary sort: Recency desc (newest first for recent conversation, but P0/P1 always win)
        items.sort_by(|a, b| {
            if a.priority != b.priority {
                a.priority.cmp(&b.priority)
            } else {
                b.created_at.cmp(&a.created_at)
            }
        });

        // Step 4: First pass — Mandatory P0 items MUST be included
        let mut p0_tokens = 0;
        for item in &items {
            if item.is_mandatory() {
                let cost = item.token_count.unwrap_or(0);
                p0_tokens += cost;
            }
        }

        if p0_tokens > budget.usable_input_budget {
            return Err(ContextError::MandatoryContextTooLarge {
                needed: p0_tokens,
                budget: budget.usable_input_budget,
            });
        }

        // Step 5: Select items within budget
        for mut item in items {
            let cost = item.token_count.unwrap_or(0);

            if item.is_mandatory() {
                item.inclusion_reason = Some(ContextInclusionReason::Mandatory);
                used_tokens += cost;
                selected.push(item);
            } else if used_tokens + cost <= budget.usable_input_budget {
                let reason = if item.priority <= 25 {
                    ContextInclusionReason::CurrentTask
                } else if item.priority <= 50 {
                    ContextInclusionReason::Recent
                } else {
                    ContextInclusionReason::Relevant
                };
                item.inclusion_reason = Some(reason);
                used_tokens += cost;
                selected.push(item);
            } else {
                excluded.push((item.id, "budget_exceeded".to_string()));
            }
        }

        Ok(SelectionResult {
            selected,
            excluded,
            total_tokens: used_tokens,
            token_count_kind: aggregate_kind,
        })
    }
}
