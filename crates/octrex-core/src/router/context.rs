use crate::models::types::ModelDescriptor;

pub const DEFAULT_SAFETY_MARGIN_TOKENS: u32 = 1024;
pub const DEFAULT_MIN_OUTPUT_TOKENS: u32 = 1024;

pub struct ContextEvaluator;

impl ContextEvaluator {
    pub fn evaluate_candidate(
        model: &ModelDescriptor,
        req_input_tokens: Option<u32>,
        req_output_tokens: Option<u32>,
    ) -> (bool, u32, u32, String) {
        let max_context = model.context_window.unwrap_or(4096);
        let input_tokens = req_input_tokens.unwrap_or(0);
        let output_tokens = req_output_tokens.unwrap_or(DEFAULT_MIN_OUTPUT_TOKENS);

        let total_required = input_tokens + output_tokens + DEFAULT_SAFETY_MARGIN_TOKENS;

        if max_context < total_required {
            (
                false,
                total_required,
                max_context,
                format!(
                    "Required context ({} tokens) exceeds model context window ({} tokens)",
                    total_required, max_context
                ),
            )
        } else {
            (
                true,
                total_required,
                max_context,
                format!(
                    "Context window capacity OK (required {} tokens <= max {} tokens)",
                    total_required, max_context
                ),
            )
        }
    }
}
