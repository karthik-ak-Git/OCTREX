use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::privacy::PrivacyContext;
use crate::tools::types::{ToolCapability, ToolId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub request_id: String,
    pub tool_id: ToolId,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub arguments: serde_json::Value,
    pub requested_capabilities: Vec<ToolCapability>,
    pub context: HashMap<String, String>,
    pub privacy_context: Option<PrivacyContext>,
    pub parent_tool_call_id: Option<String>,
    pub recursion_depth: u32,
}

impl ToolRequest {
    pub fn new(tool_id: ToolId, arguments: serde_json::Value) -> Self {
        Self {
            request_id: format!("req-{}", uuid::Uuid::new_v4().simple()),
            tool_id,
            task_id: None,
            session_id: None,
            workspace_id: None,
            arguments,
            requested_capabilities: Vec::new(),
            context: HashMap::new(),
            privacy_context: None,
            parent_tool_call_id: None,
            recursion_depth: 0,
        }
    }

    pub fn with_task_id(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_workspace_id(mut self, workspace_id: WorkspaceId) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<ToolCapability>) -> Self {
        self.requested_capabilities = capabilities;
        self
    }

    pub fn with_recursion_depth(mut self, depth: u32) -> Self {
        self.recursion_depth = depth;
        self
    }

    pub fn with_parent_call(mut self, parent_id: String) -> Self {
        self.parent_tool_call_id = Some(parent_id);
        self
    }
}
