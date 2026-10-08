use crate::events::{bus::EventBus, envelope::EventEnvelope, types::EventType};
use crate::ids::TaskId;

pub struct OrchestrationEventPublisher;

impl OrchestrationEventPublisher {
    pub fn publish_task_event(
        event_bus: &EventBus,
        event_type: EventType,
        task_id: &TaskId,
        payload: serde_json::Value,
    ) {
        let envelope = EventEnvelope::new(event_type, payload).with_task_id(task_id.clone());
        let _ = event_bus.publish(envelope);
    }
}
