use crate::context::budget::TokenBudget;
use crate::context::errors::ContextError;
use crate::context::item::ContextItem;
use crate::context::tokenizer::ContextTokenCounter;
use crate::context::types::{ContextRole, ContextSource};
use crate::privacy::PrivacyClassification;
use serde::{Deserialize, Serialize};

pub const DEFAULT_MAX_COMPACTION_ROUNDS: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    pub rounds: u32,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub items_compacted: usize,
    pub compacted_items: Vec<ContextItem>,
    pub classification: PrivacyClassification,
    pub warnings: Vec<String>,
}

pub struct CompactionEngine {
    max_rounds: u32,
}

impl Default for CompactionEngine {
    fn default() -> Self {
        Self {
            max_rounds: DEFAULT_MAX_COMPACTION_ROUNDS,
        }
    }
}

impl CompactionEngine {
    pub fn new(max_rounds: u32) -> Self {
        Self {
            max_rounds: max_rounds.max(1),
        }
    }

    /// Bounded Compaction Engine
    /// Strategy:
    ///   1. Preserve mandatory P0 system/company/security/privacy policy items
    ///   2. Preserve current user request & active task state
    ///   3. Preserve recent messages (most recent N)
    ///   4. Identify eligible older/secondary items (P3/P4 or old conversation/tool results)
    ///   5. Summarize eligible items into a CompactionSummary ContextItem
    ///   6. Recalculate tokens & repeat up to max_rounds
    /// INVARIANTS:
    ///   - Never remove mandatory policy items
    ///   - Never lower classification (summary classification = max(compacted classifications))
    ///   - Never elevate trust (summary trust = ModelGenerated)
    ///   - Compaction summary CANNOT assumed System role (role = Assistant)
    pub fn compact(
        &self,
        items: Vec<ContextItem>,
        budget: &TokenBudget,
        counter: &ContextTokenCounter,
    ) -> Result<CompactionResult, ContextError> {
        let mut current_items = items;
        for item in &mut current_items {
            if item.token_count.is_none() {
                let (count, kind) = counter.count(&item.content)?;
                item.token_count = Some(count);
                item.token_count_kind = kind;
            }
        }

        let mut round = 0;
        let tokens_before: usize = current_items
            .iter()
            .map(|i| i.token_count.unwrap_or(0))
            .sum();

        let mut warnings = Vec::new();
        let mut total_compacted_count = 0;

        while round < self.max_rounds {
            let current_tokens: usize = current_items
                .iter()
                .map(|i| i.token_count.unwrap_or(0))
                .sum();

            if current_tokens <= budget.usable_input_budget {
                // Fits within budget! No further compaction needed.
                let max_classification = current_items
                    .iter()
                    .map(|i| i.classification)
                    .max()
                    .unwrap_or(PrivacyClassification::Public);

                return Ok(CompactionResult {
                    rounds: round,
                    tokens_before,
                    tokens_after: current_tokens,
                    items_compacted: total_compacted_count,
                    compacted_items: current_items,
                    classification: max_classification,
                    warnings,
                });
            }

            round += 1;

            // Partition items into mandatory/protected vs eligible for compaction
            let mut protected = Vec::new();
            let mut eligible = Vec::new();

            let items_to_process = std::mem::take(&mut current_items);
            for item in items_to_process {
                if item.is_mandatory()
                    || item.source == ContextSource::UserRequest
                    || item.source == ContextSource::TaskState
                {
                    protected.push(item);
                } else {
                    eligible.push(item);
                }
            }

            if eligible.is_empty() {
                current_items = protected;
                warnings
                    .push("No eligible non-mandatory items available for compaction".to_string());
                break;
            }

            // Take half of the oldest eligible items to compact in this round
            let compact_batch_size = (eligible.len() / 2).max(1);
            let (to_compact, to_keep) = eligible.split_at(compact_batch_size);

            total_compacted_count += to_compact.len();

            // Compute aggregate classification of compacted items
            let summary_classification = to_compact
                .iter()
                .map(|i| i.classification)
                .max()
                .unwrap_or(PrivacyClassification::Public);

            // Build deterministic summary representation (without unconstrained LLM call)
            let mut summary_lines = Vec::new();
            summary_lines.push(format!(
                "=== CONTEXT SUMMARY (Round {}, {} items compacted) ===",
                round,
                to_compact.len()
            ));
            for item in to_compact {
                summary_lines.push(format!("- {}", item.safe_summary()));
            }
            let summary_text = summary_lines.join("\n");

            let (summary_tokens, summary_kind) = counter.count(&summary_text)?;

            let mut summary_item = ContextItem::new(
                ContextSource::CompactionSummary,
                ContextRole::Assistant, // Role safety: CompactionSummary must NOT be System
                summary_text,
            )
            .with_classification(summary_classification)
            .with_priority(60);

            summary_item.token_count = Some(summary_tokens);
            summary_item.token_count_kind = summary_kind;

            // Reconstruct item list: protected + kept + new summary item
            let mut new_items = protected;
            new_items.extend(to_keep.iter().cloned());
            new_items.push(summary_item);

            current_items = new_items;
        }

        let tokens_after: usize = current_items
            .iter()
            .map(|i| i.token_count.unwrap_or(0))
            .sum();

        let max_classification = current_items
            .iter()
            .map(|i| i.classification)
            .max()
            .unwrap_or(PrivacyClassification::Public);

        if tokens_after > budget.usable_input_budget {
            return Err(ContextError::CompactionFailed {
                rounds: round,
                reason: format!(
                    "Context still exceeds usable input budget ({} > {}) after {} rounds",
                    tokens_after, budget.usable_input_budget, round
                ),
            });
        }

        Ok(CompactionResult {
            rounds: round,
            tokens_before,
            tokens_after,
            items_compacted: total_compacted_count,
            compacted_items: current_items,
            classification: max_classification,
            warnings,
        })
    }
}
