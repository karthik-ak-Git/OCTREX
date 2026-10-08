use serde::{Deserialize, Serialize};

/// Fail-closed errors for the local runtime layer.
///
/// Local failures normalize into existing `ModelError` categories at the
/// inference boundary and never trigger cloud fallback.
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize, PartialEq)]
pub enum LocalRuntimeError {
    #[error("Local runtime '{runtime_id}' not found")]
    RuntimeNotFound { runtime_id: String },

    #[error("Local model '{model_id}' not found")]
    ModelNotFound { model_id: String },

    #[error("Duplicate local model registration: '{model_id}'")]
    DuplicateModel { model_id: String },

    #[error("Duplicate local runtime registration: '{runtime_id}'")]
    DuplicateRuntime { runtime_id: String },

    #[error("Invalid local endpoint '{endpoint}': {reason}")]
    InvalidEndpoint { endpoint: String, reason: String },

    #[error("Network boundary denied local endpoint '{endpoint}': {reason}")]
    NetworkDenied { endpoint: String, reason: String },

    #[error("Local runtime unavailable: {reason}")]
    RuntimeUnavailable { reason: String },

    #[error("Local model unavailable: {reason}")]
    ModelUnavailable { reason: String },

    #[error("Unsafe model path rejected: {reason}")]
    UnsafePath { reason: String },

    #[error("Unsupported model format '{format}'")]
    UnsupportedFormat { format: String },

    #[error("Unsafe download rejected: {reason}")]
    UnsafeDownload { reason: String },

    #[error("Insufficient resources: {reason}")]
    InsufficientResources { reason: String },

    #[error("Context budget exceeded: {reason}")]
    ContextTooLarge { reason: String },

    #[error("Request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Request cancelled: {call_id}")]
    Cancelled { call_id: String },

    #[error("Privacy gate denied local execution: {reason}")]
    PrivacyDenied { reason: String },

    #[error("Malformed runtime response: {reason}")]
    MalformedResponse { reason: String },

    #[error("Model lifecycle transition denied: {reason}")]
    LifecycleDenied { reason: String },

    #[error("Local operation failed: {reason}")]
    Internal { reason: String },
}

impl LocalRuntimeError {
    /// Normalize into the existing `ModelError` vocabulary so callers never
    /// need a second error taxonomy.
    pub fn to_model_error(&self, provider_id: &str, model_id: &str) -> crate::models::ModelError {
        use crate::models::ModelError;
        match self {
            LocalRuntimeError::RuntimeUnavailable { reason }
            | LocalRuntimeError::ModelUnavailable { reason } => ModelError::ModelUnavailable {
                model_id: model_id.to_string(),
                reason: format!("LocalRuntimeUnavailable: {}", reason),
            },
            LocalRuntimeError::Timeout { .. } => ModelError::Timeout,
            LocalRuntimeError::Cancelled { .. } => ModelError::StreamFailed {
                message: "Local inference cancelled".to_string(),
            },
            LocalRuntimeError::ContextTooLarge { reason } => ModelError::InvalidRequest {
                message: reason.clone(),
            },
            LocalRuntimeError::NetworkDenied { reason, .. }
            | LocalRuntimeError::PrivacyDenied { reason } => ModelError::ProviderUnavailable {
                provider_id: provider_id.to_string(),
                reason: reason.clone(),
            },
            LocalRuntimeError::MalformedResponse { reason } => ModelError::ProviderError {
                provider_id: provider_id.to_string(),
                message: format!("Malformed local runtime response: {}", reason),
            },
            other => ModelError::ProviderError {
                provider_id: provider_id.to_string(),
                message: format!("Local runtime error: {}", other),
            },
        }
    }
}
