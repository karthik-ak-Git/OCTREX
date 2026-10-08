use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Memory validation failed: {reason}")]
    Validation { reason: String },

    #[error("Memory not found: {memory_id}")]
    NotFound { memory_id: String },

    #[error("Memory access denied: {reason}")]
    AccessDenied { reason: String },

    #[error("Memory security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Memory persistence failure: {message}")]
    Persistence { message: String },

    #[error("Internal memory error: {message}")]
    Internal { message: String },
}

impl From<MemoryError> for crate::error::OctrexError {
    fn from(e: MemoryError) -> Self {
        match &e {
            MemoryError::Validation { .. } => crate::error::OctrexError::Validation {
                message: e.to_string(),
                details: None,
            },
            MemoryError::NotFound { memory_id } => crate::error::OctrexError::NotFound {
                resource: format!("Memory '{}'", memory_id),
            },
            MemoryError::AccessDenied { .. } | MemoryError::SecurityViolation { .. } => {
                crate::error::OctrexError::PermissionDenied {
                    reason: e.to_string(),
                }
            }
            MemoryError::Persistence { .. } | MemoryError::Internal { .. } => {
                crate::error::OctrexError::Internal {
                    message: e.to_string(),
                }
            }
        }
    }
}
