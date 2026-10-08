use crate::events::types::EventType;
use crate::ids::{EventId, RequestId, SessionId, TaskId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: EventType,
    pub timestamp: u64,
    pub request_id: Option<RequestId>,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    pub fn new<T: Serialize>(event_type: EventType, payload: T) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            event_id: EventId::new(),
            event_type,
            timestamp,
            request_id: None,
            task_id: None,
            session_id: None,
            workspace_id: None,
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(request_id);
        self
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
}
