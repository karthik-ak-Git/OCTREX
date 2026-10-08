use crate::memory::types::{now_millis, MemoryItem, MemoryType};

/// Retention policy: TemporaryFact short-lived; TaskFact until task lifecycle
/// ends + 7 days; Workspace/Project/User durable until changed/deleted.
pub fn default_expiry(mem_type: MemoryType, created_at: u64) -> Option<u64> {
    const HOUR: u64 = 3600_000;
    const DAY: u64 = 24 * HOUR;
    match mem_type {
        MemoryType::TemporaryFact => Some(created_at + HOUR),
        MemoryType::TaskFact => Some(created_at + 7 * DAY),
        MemoryType::WorkflowFact | MemoryType::SkillFact | MemoryType::ArtifactFact => {
            Some(created_at + 30 * DAY)
        }
        MemoryType::UserPreference
        | MemoryType::ProjectFact
        | MemoryType::WorkspaceFact
        | MemoryType::Decision
        | MemoryType::Constraint
        | MemoryType::Procedure => None,
    }
}

pub fn is_expired(item: &MemoryItem, now: u64) -> bool {
    item.expires_at.map(|exp| now > exp).unwrap_or(false)
}

pub fn apply_default_retention(item: &mut MemoryItem) {
    if item.expires_at.is_none() {
        item.expires_at = default_expiry(item.mem_type, item.created_at);
    }
}

pub fn now() -> u64 {
    now_millis()
}
