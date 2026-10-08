use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Verification input invalid: {reason}")]
    InvalidInput { reason: String },

    #[error("Task '{task_id}' not found")]
    TaskNotFound { task_id: String },

    #[error("Workspace '{workspace_id}' not found")]
    WorkspaceNotFound { workspace_id: String },

    #[error("Verification run '{verification_id}' not found")]
    RunNotFound { verification_id: String },

    #[error("Filesystem boundary denied verification probe: {reason}")]
    FilesystemDenied { reason: String },

    #[error("Persistence failure: {message}")]
    Persistence { message: String },

    #[error("Repair budget exhausted for task '{task_id}' ({attempts}/{max} attempts)")]
    RepairBudgetExhausted {
        task_id: String,
        attempts: u32,
        max: u32,
    },

    #[error("Internal verification error: {message}")]
    Internal { message: String },
}

impl From<VerificationError> for crate::error::OctrexError {
    fn from(e: VerificationError) -> Self {
        match &e {
            VerificationError::TaskNotFound { .. } | VerificationError::RunNotFound { .. } => {
                crate::error::OctrexError::NotFound {
                    resource: e.to_string(),
                }
            }
            VerificationError::InvalidInput { .. } => crate::error::OctrexError::Validation {
                message: e.to_string(),
                details: None,
            },
            VerificationError::FilesystemDenied { reason } => {
                crate::error::OctrexError::PermissionDenied {
                    reason: reason.clone(),
                }
            }
            VerificationError::RepairBudgetExhausted { .. } => {
                crate::error::OctrexError::VerificationFailed {
                    check_name: "repair_budget".to_string(),
                }
            }
            VerificationError::Persistence { .. } | VerificationError::Internal { .. } => {
                crate::error::OctrexError::Internal {
                    message: e.to_string(),
                }
            }
            VerificationError::WorkspaceNotFound { .. } => crate::error::OctrexError::NotFound {
                resource: e.to_string(),
            },
        }
    }
}
