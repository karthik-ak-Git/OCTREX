use thiserror::Error;

#[derive(Debug, Error, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum ContextError {
    #[error(
        "Context budget exceeded: requested {needed} tokens, available input budget is {available}"
    )]
    BudgetExceeded { needed: usize, available: usize },

    #[error("Tokenizer unavailable for model '{model_id}'")]
    TokenizerUnavailable { model_id: String },

    #[error("Tokenizer unknown for model '{model_id}' — conservative fallback applied")]
    TokenizerUnknown { model_id: String },

    #[error("Context item '{item_id}' too large ({tokens} tokens) for available context budget")]
    ItemTooLarge { item_id: String, tokens: usize },

    #[error("Compaction failed after {rounds} rounds: {reason}")]
    CompactionFailed { rounds: u32, reason: String },

    #[error("Invalid context role: untrusted source '{source_name}' cannot assume privileged role '{role}'")]
    InvalidContextRole { source_name: String, role: String },

    #[error("Classification conflict: cannot downgrade classification from {source_classification} to {target_classification}")]
    ClassificationConflict {
        source_classification: String,
        target_classification: String,
    },

    #[error("Cross-session context isolation violation: item from session '{item_session}' cannot enter session '{target_session}'")]
    IsolationViolation {
        item_session: String,
        target_session: String,
    },

    #[error("Context checkpoint invalid or incompatible: {reason}")]
    CheckpointInvalid { reason: String },

    #[error("Context source unavailable: {source_name}")]
    SourceUnavailable { source_name: String },

    #[error("Context policy violation: {reason}")]
    PolicyViolation { reason: String },

    #[error("Mandatory policy context consumes {needed} tokens, which exceeds the total input budget of {budget}")]
    MandatoryContextTooLarge { needed: usize, budget: usize },

    #[error("Model descriptor not found for '{model_id}'")]
    ModelNotFound { model_id: String },

    #[error("Internal context engine error: {message}")]
    Internal { message: String },
}
