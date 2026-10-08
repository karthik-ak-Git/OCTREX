use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OrchestrationError {
    #[error("Invalid state transition from '{from}' to '{to}'")]
    InvalidStateTransition { from: String, to: String },

    #[error("Invalid task plan: {reason}")]
    InvalidPlan { reason: String },

    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Boundary violation: {reason}")]
    BoundaryViolation { reason: String },

    #[error(
        "Execution limit exceeded: {limit_type} (limit: {limit_value}, actual: {actual_value})"
    )]
    LimitExceeded {
        limit_type: String,
        limit_value: u64,
        actual_value: u64,
    },

    #[error("Non-retryable error: {reason}")]
    NonRetryableError { reason: String },

    #[error("User consent or input required: {prompt}")]
    UserConsentRequired { prompt: String },

    #[error("Verification failed for check '{check_name}': {reason}")]
    VerificationFailed { check_name: String, reason: String },

    #[error("Task execution was cancelled")]
    CancellationRequested,

    #[error("Task step failed: {step_id} - {reason}")]
    StepFailed { step_id: String, reason: String },

    #[error("Internal orchestration error: {message}")]
    Internal { message: String },
}

impl From<crate::error::OctrexError> for OrchestrationError {
    fn from(err: crate::error::OctrexError) -> Self {
        OrchestrationError::Internal {
            message: err.to_string(),
        }
    }
}

impl From<OrchestrationError> for crate::error::AppErrorResponse {
    fn from(err: OrchestrationError) -> Self {
        let code = match &err {
            OrchestrationError::InvalidStateTransition { .. } => {
                crate::error::ErrorCode::ValidationError
            }
            OrchestrationError::InvalidPlan { .. } => crate::error::ErrorCode::ValidationError,
            OrchestrationError::SecurityViolation { .. } => crate::error::ErrorCode::PolicyDenied,
            OrchestrationError::BoundaryViolation { .. } => {
                crate::error::ErrorCode::FilesystemError
            }
            OrchestrationError::LimitExceeded { .. } => crate::error::ErrorCode::ContextOverflow,
            OrchestrationError::NonRetryableError { .. } => crate::error::ErrorCode::InternalError,
            OrchestrationError::UserConsentRequired { .. } => {
                crate::error::ErrorCode::PermissionDenied
            }
            OrchestrationError::VerificationFailed { .. } => {
                crate::error::ErrorCode::VerificationFailed
            }
            OrchestrationError::CancellationRequested => crate::error::ErrorCode::Cancelled,
            OrchestrationError::StepFailed { .. } => crate::error::ErrorCode::ToolFailed,
            OrchestrationError::Internal { .. } => crate::error::ErrorCode::InternalError,
        };

        crate::error::AppErrorResponse {
            code,
            message: err.to_string(),
            details: None,
            correlation_id: None,
        }
    }
}
