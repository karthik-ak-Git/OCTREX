use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use crate::models::capabilities::ModelCapability;
use crate::privacy::{PrivacyClassification, PrivacyInput};
use crate::router::types::RoutingMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub request_id: RequestId,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,

    pub purpose: String,
    pub routing_mode: RoutingMode,
    pub user_selected_model: Option<String>,

    pub required_capabilities: Vec<ModelCapability>,
    pub required_context_tokens: Option<u32>,
    pub required_output_tokens: Option<u32>,

    pub latency_requirement: Option<String>,
    pub quality_requirement: Option<String>,
    pub cost_requirement: Option<String>,

    pub privacy_classification: Option<PrivacyClassification>,

    pub allow_local: bool,
    pub allow_on_prem: bool,
    pub allow_online: bool,

    pub inputs: Vec<PrivacyInput>,
}

impl RoutingRequest {
    pub fn new(purpose: impl Into<String>) -> Self {
        Self {
            request_id: RequestId::new(),
            task_id: None,
            session_id: None,
            workspace_id: None,
            purpose: purpose.into(),
            routing_mode: RoutingMode::Auto,
            user_selected_model: None,
            required_capabilities: Vec::new(),
            required_context_tokens: None,
            required_output_tokens: None,
            latency_requirement: None,
            quality_requirement: None,
            cost_requirement: None,
            privacy_classification: None,
            allow_local: true,
            allow_on_prem: true,
            allow_online: true,
            inputs: Vec::new(),
        }
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = request_id;
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

    pub fn with_routing_mode(mut self, mode: RoutingMode) -> Self {
        self.routing_mode = mode;
        self
    }

    pub fn with_user_selected_model(mut self, model_id: impl Into<String>) -> Self {
        self.user_selected_model = Some(model_id.into());
        self.routing_mode = RoutingMode::ExplicitModel;
        self
    }

    pub fn with_capability(mut self, cap: ModelCapability) -> Self {
        if !self.required_capabilities.contains(&cap) {
            self.required_capabilities.push(cap);
        }
        self
    }

    pub fn with_required_context(mut self, context_tokens: u32, output_tokens: u32) -> Self {
        self.required_context_tokens = Some(context_tokens);
        self.required_output_tokens = Some(output_tokens);
        self
    }

    pub fn with_privacy_classification(mut self, classification: PrivacyClassification) -> Self {
        self.privacy_classification = Some(classification);
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<PrivacyInput>) -> Self {
        self.inputs = inputs;
        self
    }
}
