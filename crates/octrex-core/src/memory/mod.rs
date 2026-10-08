pub mod classifier;
pub mod errors;
pub mod extractor;
pub mod policy;
pub mod retention;
pub mod retrieval;
pub mod service;
pub mod store;
pub mod types;

pub mod events {
    use crate::events::{EventBus, EventEnvelope, EventType};

    pub fn publish_memory_event(bus: &EventBus, event_type: EventType, memory_id: &str) {
        let _ = bus.publish(EventEnvelope::new(
            event_type,
            serde_json::json!({ "memory_id": memory_id }),
        ));
    }
}

pub mod provenance {
    use crate::memory::types::{MemoryItem, MemorySource};

    /// Never expose sensitive provenance unnecessarily.
    pub fn safe_provenance(item: &MemoryItem) -> serde_json::Value {
        let actor = if item.classification == crate::privacy::PrivacyClassification::Secret {
            "[REDACTED]".to_string()
        } else {
            item.provenance.actor.clone()
        };
        serde_json::json!({
            "source": item.source.to_string(),
            "actor": actor,
            "tool_id": item.provenance.tool_id,
            "model_id": item.provenance.model_id,
            "imported_from": item.provenance.imported_from,
            "untrusted": matches!(item.source, MemorySource::ModelDerived | MemorySource::Imported | MemorySource::ToolDerived),
        })
    }
}

pub mod ranking {
    pub use crate::memory::retrieval::rank_items;
}

pub use classifier::{classification_floor, detect_classification_floor, enforce_no_downgrade};
pub use errors::MemoryError;
pub use extractor::{ExtractionInput, MemoryExtractor};
pub use policy::{safe_memory_summary, MemoryPolicy};
pub use retention::{apply_default_retention, default_expiry, is_expired};
pub use service::MemoryService;
pub use store::MemoryStore;
pub use types::{
    now_millis, MemoryCandidate, MemoryItem, MemoryProvenance, MemoryQuery, MemoryResult,
    MemoryScope, MemorySource, MemoryType,
};

#[cfg(test)]
mod tests;
