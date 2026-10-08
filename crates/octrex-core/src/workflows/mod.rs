pub mod errors;
pub mod executor;
pub mod planner;
pub mod policy;
pub mod registry;
pub mod service;
pub mod types;
pub mod validator;

pub mod events {
    use crate::events::{EventBus, EventEnvelope, EventType};

    pub fn publish_workflow_event(bus: &EventBus, event_type: EventType, workflow_id: &str) {
        let _ = bus.publish(EventEnvelope::new(
            event_type,
            serde_json::json!({ "workflow_id": workflow_id }),
        ));
    }
}

pub use errors::WorkflowError;
pub use executor::WorkflowExecutor;
pub use planner::WorkflowPlanner;
pub use registry::WorkflowRegistry;
pub use service::WorkflowService;
pub use types::{
    now_millis, WorkflowDefinition, WorkflowRun, WorkflowRunStatus, WorkflowStatus, WorkflowStep,
    WorkflowStepKind,
};
pub use validator::{validate_workflow, MAX_WORKFLOW_DEPTH, MAX_WORKFLOW_STEPS};

#[cfg(test)]
mod tests;
