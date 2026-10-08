use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize, PartialEq)]
pub enum RouterError {
    #[error("Routing denied by policy: {reason}")]
    PolicyDenied { reason: String },

    #[error("No compatible model available: {reason}")]
    NoCompatibleModel { reason: String },

    #[error("Explicitly selected model '{model_id}' is invalid or unavailable: {reason}")]
    InvalidUserSelection { model_id: String, reason: String },

    #[error("Hardware incompatible with model requirement: {details}")]
    HardwareIncompatible { details: String },

    #[error("Context limit exceeded: required {required} tokens exceeds max {max}")]
    ContextTooLarge { required: u32, max: u32 },

    #[error("Provider '{provider_id}' is unavailable")]
    ProviderUnavailable { provider_id: String },

    #[error("Internal router error: {message}")]
    InternalError { message: String },
}
