use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Workflow validation failed: {reason}")]
    Validation { reason: String },

    #[error("Workflow not found: {workflow_id}")]
    NotFound { workflow_id: String },

    #[error("Workflow is not eligible: {reason}")]
    NotEligible { reason: String },

    #[error("Workflow security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Workflow capability denied: {reason}")]
    CapabilityDenied { reason: String },

    #[error("Workflow persistence failure: {message}")]
    Persistence { message: String },

    #[error("Internal workflow error: {message}")]
    Internal { message: String },
}

impl From<WorkflowError> for crate::error::OctrexError {
    fn from(e: WorkflowError) -> Self {
        match &e {
            WorkflowError::Validation { .. } => crate::error::OctrexError::Validation {
                message: e.to_string(),
                details: None,
            },
            WorkflowError::NotFound { workflow_id } => crate::error::OctrexError::NotFound {
                resource: format!("Workflow '{}'", workflow_id),
            },
            WorkflowError::NotEligible { .. }
            | WorkflowError::SecurityViolation { .. }
            | WorkflowError::CapabilityDenied { .. } => {
                crate::error::OctrexError::PermissionDenied {
                    reason: e.to_string(),
                }
            }
            WorkflowError::Persistence { .. } | WorkflowError::Internal { .. } => {
                crate::error::OctrexError::Internal {
                    message: e.to_string(),
                }
            }
        }
    }
}
