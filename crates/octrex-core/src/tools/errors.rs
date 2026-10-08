use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolError {
    ToolNotFound(String),
    ToolDisabled(String),
    InvalidArguments(String),
    CapabilityDenied { capability: String, reason: String },
    UnknownCapability(String),
    PermissionDenied(String),
    PrivacyDenied(String),
    NetworkDenied(String),
    FilesystemDenied(String),
    ConsentRequired(String),
    ConsentDenied(String),
    Timeout(u64),
    Cancelled(String),
    RecursionLimit(u32),
    ConcurrencyLimit(u32),
    OutputTooLarge { size: usize, limit: usize },
    InputTooLarge { size: usize, limit: usize },
    McpUnavailable(String),
    McpDenied(String),
    ToolExecutionFailed(String),
    UnknownToolState(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::ToolNotFound(id) => write!(f, "Tool '{}' not found in registry", id),
            ToolError::ToolDisabled(id) => write!(f, "Tool '{}' is disabled by policy", id),
            ToolError::InvalidArguments(msg) => write!(f, "Invalid tool arguments: {}", msg),
            ToolError::CapabilityDenied { capability, reason } => {
                write!(f, "Capability '{}' denied: {}", capability, reason)
            }
            ToolError::UnknownCapability(cap) => write!(f, "Unknown capability: {}", cap),
            ToolError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ToolError::PrivacyDenied(msg) => {
                write!(f, "Privacy policy denied tool execution: {}", msg)
            }
            ToolError::NetworkDenied(msg) => {
                write!(f, "Network policy denied tool execution: {}", msg)
            }
            ToolError::FilesystemDenied(msg) => {
                write!(f, "Filesystem boundary denied tool execution: {}", msg)
            }
            ToolError::ConsentRequired(msg) => write!(f, "User consent required: {}", msg),
            ToolError::ConsentDenied(msg) => write!(f, "User consent denied: {}", msg),
            ToolError::Timeout(ms) => write!(f, "Tool execution timed out after {}ms", ms),
            ToolError::Cancelled(msg) => write!(f, "Tool execution cancelled: {}", msg),
            ToolError::RecursionLimit(depth) => {
                write!(f, "Tool recursion limit reached (max depth: {})", depth)
            }
            ToolError::ConcurrencyLimit(max) => write!(
                f,
                "Tool concurrency limit reached (max concurrent: {})",
                max
            ),
            ToolError::OutputTooLarge { size, limit } => write!(
                f,
                "Tool output size ({} bytes) exceeds limit ({} bytes)",
                size, limit
            ),
            ToolError::InputTooLarge { size, limit } => write!(
                f,
                "Tool input size ({} bytes) exceeds limit ({} bytes)",
                size, limit
            ),
            ToolError::McpUnavailable(msg) => write!(f, "MCP server unavailable: {}", msg),
            ToolError::McpDenied(msg) => write!(f, "MCP execution denied: {}", msg),
            ToolError::ToolExecutionFailed(msg) => write!(f, "Tool execution failed: {}", msg),
            ToolError::UnknownToolState(msg) => write!(f, "Unknown tool state: {}", msg),
        }
    }
}

impl std::error::Error for ToolError {}
