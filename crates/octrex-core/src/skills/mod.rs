pub mod errors;
pub mod executor;
pub mod matcher;
pub mod policy;
pub mod provenance;
pub mod registry;
pub mod security;
pub mod service;
pub mod types;
pub mod validator;
pub mod versioning;

pub mod events {
    use crate::events::{EventBus, EventEnvelope, EventType};

    pub fn publish_skill_event(bus: &EventBus, event_type: EventType, skill_id: &str) {
        let _ = bus.publish(EventEnvelope::new(
            event_type,
            serde_json::json!({ "skill_id": skill_id }),
        ));
    }
}

pub use errors::SkillError;
pub use executor::SkillExecutor;
pub use matcher::{RankedSkill, SkillMatchRequest, SkillMatcher};
pub use policy::{check_eligibility, check_policy_compatibility};
pub use registry::SkillRegistry;
pub use service::{preferred_free_local_model, SkillService};
pub use types::{
    now_millis, SkillDefinition, SkillMetrics, SkillProvenance, SkillSource, SkillStatus,
    SkillStep, SkillStepKind,
};
pub use validator::{post_validation_status, validate_skill, validate_version};
pub use versioning::{is_newer, parse_version, parse_versioned_id, versioned_id};

#[cfg(test)]
mod tests;
