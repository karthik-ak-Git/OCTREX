use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum FilesystemError {
    #[error("Path traversal detected in path '{path}': {reason}")]
    PathTraversalDetected { path: String, reason: String },

    #[error("Target path '{path}' escapes trusted workspace boundary '{root}'")]
    OutsideWorkspaceBoundary { path: String, root: String },

    #[error("Symlink escape detected for path '{path}': target '{target}' resolves outside workspace boundary")]
    SymlinkEscapeDetected { path: String, target: String },

    #[error("Reparse point or junction escape detected for path '{path}'")]
    ReparseOrJunctionEscape { path: String },

    #[error("Special filesystem node blocked at '{path}': {node_type}")]
    SpecialNodeBlocked { path: String, node_type: String },

    #[error("Workspace '{workspace_id}' is read-only. Modification operations are prohibited")]
    ReadOnlyWorkspace { workspace_id: String },

    #[error("Protected path rule '{pattern}' blocked access to '{path}': {reason}")]
    ProtectedPathBlocked {
        path: String,
        pattern: String,
        reason: String,
    },

    #[error(
        "File operation on '{path}' exceeds size limit ({size_bytes} bytes > {limit_bytes} bytes)"
    )]
    FileTooLarge {
        path: String,
        size_bytes: u64,
        limit_bytes: u64,
    },

    #[error("Filesystem operation '{operation}' blocked: {reason}")]
    OperationBlocked { operation: String, reason: String },

    #[error("Workspace '{workspace_id}' not found in backend repository")]
    WorkspaceNotFound { workspace_id: String },

    #[error("Filesystem IO error: {message}")]
    IOError { message: String },

    #[error("Privacy policy blocked file operation: {reason}")]
    PrivacyPolicyBlocked { reason: String },

    #[error("Network security boundary blocked filesystem export: {reason}")]
    NetworkPolicyBlocked { reason: String },

    #[error("Confirmation required for operation '{operation}' on path '{path}' (Decision ID: {decision_id})")]
    ConfirmationRequired {
        decision_id: String,
        operation: String,
        path: String,
    },

    #[error("Invalid path format or non-canonical path: '{path}'")]
    InvalidPath { path: String },
}

impl From<std::io::Error> for FilesystemError {
    fn from(err: std::io::Error) -> Self {
        FilesystemError::IOError {
            message: err.to_string(),
        }
    }
}
