use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::tools::capability::CapabilityGrant;
use crate::tools::types::{CapabilityScope, ToolCapability, ToolId};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ToolExecutionContext {
    pub tool_id: ToolId,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub workspace_path: Option<PathBuf>,
    pub granted_capabilities: Vec<ToolCapability>,
    pub scope: CapabilityScope,
    pub timeout_ms: u64,
    pub is_cancelled: bool,
}

impl ToolExecutionContext {
    pub fn from_grant(
        grant: &CapabilityGrant,
        workspace_path: Option<PathBuf>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            tool_id: grant.tool_id.clone(),
            task_id: grant.task_id.clone(),
            session_id: grant.session_id.clone(),
            workspace_id: grant.workspace_id.clone(),
            workspace_path,
            granted_capabilities: grant.capabilities.clone(),
            scope: grant.scope.clone(),
            timeout_ms,
            is_cancelled: false,
        }
    }

    pub fn has_capability(&self, cap: &ToolCapability) -> bool {
        self.granted_capabilities.contains(cap)
    }

    pub fn check_cancellation(&self) -> Result<(), String> {
        if self.is_cancelled {
            Err("Tool execution was cancelled by user".to_string())
        } else {
            Ok(())
        }
    }
}
