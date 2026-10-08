use crate::ids::RequestId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationError,
    NotFound,
    Conflict,
    PermissionDenied,
    PolicyDenied,
    PrivacyBlocked,
    NetworkBlocked,
    ModelUnavailable,
    HardwareIncompatible,
    ContextOverflow,
    ToolFailed,
    VerificationFailed,
    FilesystemError,
    ProcessError,
    McpError,
    ProviderError,
    Timeout,
    Cancelled,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::ValidationError => "VALIDATION_ERROR",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::PermissionDenied => "PERMISSION_DENIED",
            ErrorCode::PolicyDenied => "POLICY_DENIED",
            ErrorCode::PrivacyBlocked => "PRIVACY_BLOCKED",
            ErrorCode::NetworkBlocked => "NETWORK_BLOCKED",
            ErrorCode::ModelUnavailable => "MODEL_UNAVAILABLE",
            ErrorCode::HardwareIncompatible => "HARDWARE_INCOMPATIBLE",
            ErrorCode::ContextOverflow => "CONTEXT_OVERFLOW",
            ErrorCode::ToolFailed => "TOOL_FAILED",
            ErrorCode::VerificationFailed => "VERIFICATION_FAILED",
            ErrorCode::FilesystemError => "FILESYSTEM_ERROR",
            ErrorCode::ProcessError => "PROCESS_ERROR",
            ErrorCode::McpError => "MCP_ERROR",
            ErrorCode::ProviderError => "PROVIDER_ERROR",
            ErrorCode::Timeout => "TIMEOUT",
            ErrorCode::Cancelled => "CANCELLED",
            ErrorCode::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Error)]
pub enum OctrexError {
    #[error("Validation Error: {message}")]
    Validation {
        message: String,
        details: Option<serde_json::Value>,
    },

    #[error("Not Found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Permission Denied: {reason}")]
    PermissionDenied { reason: String },

    #[error("Policy Denied: {policy}")]
    PolicyDenied { policy: String },

    #[error("Privacy Blocked: {reason}")]
    PrivacyBlocked { reason: String },

    #[error("Network Blocked: {endpoint}")]
    NetworkBlocked { endpoint: String },

    #[error("Model Unavailable: {model_id}")]
    ModelUnavailable { model_id: String },

    #[error("Hardware Incompatible: {reason}")]
    HardwareIncompatible { reason: String },

    #[error("Context Overflow: {message}")]
    ContextOverflow { message: String },

    #[error("Tool Failed: {tool_name} - {error}")]
    ToolFailed { tool_name: String, error: String },

    #[error("Verification Failed: {check_name}")]
    VerificationFailed { check_name: String },

    #[error("Filesystem Error: {message}")]
    Filesystem { message: String },

    #[error("Process Error: {message}")]
    Process { message: String },

    #[error("MCP Error: {message}")]
    Mcp { message: String },

    #[error("Provider Error ({provider_id}): {message}")]
    Provider {
        provider_id: String,
        message: String,
    },

    #[error("Operation Timed Out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Operation Cancelled")]
    Cancelled,

    #[error("Internal Engine Error: {message}")]
    Internal { message: String },

    #[error("Orchestration Error: {message}")]
    Orchestration { message: String },
}

impl From<crate::orchestration::OrchestrationError> for OctrexError {
    fn from(err: crate::orchestration::OrchestrationError) -> Self {
        OctrexError::Orchestration {
            message: err.to_string(),
        }
    }
}

impl OctrexError {
    pub fn code(&self) -> ErrorCode {
        match self {
            OctrexError::Validation { .. } => ErrorCode::ValidationError,
            OctrexError::NotFound { .. } => ErrorCode::NotFound,
            OctrexError::Conflict { .. } => ErrorCode::Conflict,
            OctrexError::PermissionDenied { .. } => ErrorCode::PermissionDenied,
            OctrexError::PolicyDenied { .. } => ErrorCode::PolicyDenied,
            OctrexError::PrivacyBlocked { .. } => ErrorCode::PrivacyBlocked,
            OctrexError::NetworkBlocked { .. } => ErrorCode::NetworkBlocked,
            OctrexError::ModelUnavailable { .. } => ErrorCode::ModelUnavailable,
            OctrexError::HardwareIncompatible { .. } => ErrorCode::HardwareIncompatible,
            OctrexError::ContextOverflow { .. } => ErrorCode::ContextOverflow,
            OctrexError::ToolFailed { .. } => ErrorCode::ToolFailed,
            OctrexError::VerificationFailed { .. } => ErrorCode::VerificationFailed,
            OctrexError::Filesystem { .. } => ErrorCode::FilesystemError,
            OctrexError::Process { .. } => ErrorCode::ProcessError,
            OctrexError::Mcp { .. } => ErrorCode::McpError,
            OctrexError::Provider { .. } => ErrorCode::ProviderError,
            OctrexError::Timeout { .. } => ErrorCode::Timeout,
            OctrexError::Cancelled => ErrorCode::Cancelled,
            OctrexError::Internal { .. } => ErrorCode::InternalError,
            OctrexError::Orchestration { .. } => ErrorCode::InternalError,
        }
    }

    pub fn to_response(&self, correlation_id: Option<RequestId>) -> AppErrorResponse {
        AppErrorResponse {
            code: self.code(),
            message: self.to_string(),
            details: match self {
                OctrexError::Validation { details, .. } => details.clone(),
                _ => None,
            },
            correlation_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub correlation_id: Option<RequestId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization_and_code() {
        let err = OctrexError::NotFound {
            resource: "Workspace /tmp/foo".to_string(),
        };
        assert_eq!(err.code(), ErrorCode::NotFound);

        let req_id = RequestId::new();
        let resp = err.to_response(Some(req_id.clone()));
        assert_eq!(resp.code, ErrorCode::NotFound);
        assert_eq!(resp.correlation_id, Some(req_id));

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("NOT_FOUND"));
    }
}
