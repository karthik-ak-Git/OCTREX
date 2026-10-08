use super::endpoint::NetworkEndpoint;
use super::types::NetworkCapability;
use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub request_id: RequestId,
    pub source: String,
    pub capability: NetworkCapability,
    pub endpoint: NetworkEndpoint,
    pub method: String,
    pub headers_metadata: Vec<String>,
    pub payload_summary: Option<String>,
    pub privacy_decision: Option<String>,
    pub workspace_id: Option<WorkspaceId>,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
}

impl NetworkRequest {
    pub fn new(
        source: impl Into<String>,
        capability: NetworkCapability,
        endpoint: NetworkEndpoint,
    ) -> Self {
        Self {
            request_id: RequestId::new(),
            source: source.into(),
            capability,
            endpoint,
            method: "POST".to_string(),
            headers_metadata: Vec::new(),
            payload_summary: None,
            privacy_decision: None,
            workspace_id: None,
            session_id: None,
            task_id: None,
            provider_id: None,
            model_id: None,
        }
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into().to_uppercase();
        self
    }

    pub fn with_task_context(
        mut self,
        workspace_id: Option<WorkspaceId>,
        session_id: Option<SessionId>,
        task_id: Option<TaskId>,
    ) -> Self {
        self.workspace_id = workspace_id;
        self.session_id = session_id;
        self.task_id = task_id;
        self
    }

    pub fn with_provider_info(
        mut self,
        provider_id: Option<String>,
        model_id: Option<String>,
    ) -> Self {
        self.provider_id = provider_id;
        self.model_id = model_id;
        self
    }

    pub fn with_privacy_decision(mut self, privacy_decision: Option<String>) -> Self {
        self.privacy_decision = privacy_decision;
        self
    }
}
