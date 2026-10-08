use crate::events::envelope::EventEnvelope;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, envelope: EventEnvelope) -> Result<usize, String> {
        self.sender
            .send(envelope)
            .map_err(|e| format!("Failed to publish event: {}", e))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::EventType;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        let env = EventEnvelope::new(
            EventType::ApplicationReady,
            serde_json::json!({ "ready": true }),
        );
        bus.publish(env.clone()).expect("Should publish");

        let recv = rx.recv().await.expect("Should receive");
        assert_eq!(recv.event_id, env.event_id);
        assert_eq!(recv.event_type, EventType::ApplicationReady);
    }
}
