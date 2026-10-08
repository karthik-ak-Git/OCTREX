use crate::context::errors::ContextError;
use crate::context::types::TokenCountKind;
use serde::{Deserialize, Serialize};

pub const DEFAULT_OUTPUT_RESERVE_RATIO: f32 = 0.25; // 25% default reserve for output
pub const DEFAULT_SAFETY_MARGIN_RATIO: f32 = 0.05; // 5% safety margin
pub const DEFAULT_FIXED_OVERHEAD: usize = 64; // Token overhead for system formatting

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetPolicy {
    pub output_reserve_ratio: f32,
    pub safety_margin_ratio: f32,
    pub fixed_overhead: usize,
}

impl Default for BudgetPolicy {
    fn default() -> Self {
        Self {
            output_reserve_ratio: DEFAULT_OUTPUT_RESERVE_RATIO,
            safety_margin_ratio: DEFAULT_SAFETY_MARGIN_RATIO,
            fixed_overhead: DEFAULT_FIXED_OVERHEAD,
        }
    }
}

/// Deterministic Token Budget Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub model_id: String,
    pub context_window: usize,
    pub reserved_output: usize,
    pub safety_margin: usize,
    pub fixed_overhead: usize,
    pub usable_input_budget: usize,
    pub current_usage: usize,
    pub token_count_kind: TokenCountKind,
}

impl TokenBudget {
    pub fn compute(
        model_id: impl Into<String>,
        context_window: usize,
        max_output_tokens: Option<usize>,
        policy: &BudgetPolicy,
    ) -> Result<Self, ContextError> {
        let m_id = model_id.into();
        if context_window == 0 {
            return Err(ContextError::Internal {
                message: format!("Model '{}' has 0 context window", m_id),
            });
        }

        // Output reservation: explicit max_output_tokens or policy ratio (capped at 50% window)
        let reserved_output = max_output_tokens
            .unwrap_or_else(|| (context_window as f32 * policy.output_reserve_ratio) as usize)
            .min(context_window / 2)
            .max(256);

        let safety_margin = (context_window as f32 * policy.safety_margin_ratio) as usize;
        let fixed_overhead = policy.fixed_overhead;

        let total_reserved = reserved_output + safety_margin + fixed_overhead;
        if total_reserved >= context_window {
            return Err(ContextError::BudgetExceeded {
                needed: total_reserved,
                available: context_window,
            });
        }

        let usable_input_budget = context_window - total_reserved;

        let budget = Self {
            model_id: m_id,
            context_window,
            reserved_output,
            safety_margin,
            fixed_overhead,
            usable_input_budget,
            current_usage: 0,
            token_count_kind: TokenCountKind::Unknown,
        };

        debug_assert!(budget.validate_invariant());
        Ok(budget)
    }

    pub fn remaining(&self) -> usize {
        self.usable_input_budget.saturating_sub(self.current_usage)
    }

    pub fn utilization_percent(&self) -> f32 {
        if self.usable_input_budget == 0 {
            return 100.0;
        }
        (self.current_usage as f32 / self.usable_input_budget as f32) * 100.0
    }

    pub fn would_overflow(&self, additional_tokens: usize) -> bool {
        self.current_usage + additional_tokens > self.usable_input_budget
    }

    /// Hard invariant check: usable_input_budget + reserved_output + safety_margin + fixed_overhead <= context_window
    pub fn validate_invariant(&self) -> bool {
        self.usable_input_budget + self.reserved_output + self.safety_margin + self.fixed_overhead
            <= self.context_window
    }
}
