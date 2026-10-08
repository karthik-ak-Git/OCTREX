use crate::memory::errors::MemoryError;
use crate::memory::types::{MemoryItem, MemorySource, MemoryType};
use crate::privacy::PrivacyClassification;

/// Classification must never be downgraded during summarization, extraction,
/// retrieval, or compaction. This helper computes the floor (max) of two
/// classifications and refuses to lower it.
pub fn classification_floor(
    a: PrivacyClassification,
    b: PrivacyClassification,
) -> PrivacyClassification {
    if (a as u8) >= (b as u8) {
        a
    } else {
        b
    }
}

/// Refuse to store content whose classification is below detected signals.
/// Minimal detector: secrets/credentials/tokens raise the floor to SECRET.
pub fn detect_classification_floor(content: &str) -> PrivacyClassification {
    let lower = content.to_lowercase();
    if lower.contains("api_key")
        || lower.contains("api-key")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("private_key")
        || lower.contains("private-key")
        || lower.contains("bearer ")
    {
        PrivacyClassification::Secret
    } else if lower.contains("internal")
        || lower.contains("confidential")
        || lower.contains("restricted")
    {
        PrivacyClassification::Confidential
    } else {
        PrivacyClassification::Public
    }
}

pub fn enforce_no_downgrade(
    declared: PrivacyClassification,
    content: &str,
) -> Result<PrivacyClassification, MemoryError> {
    let floor = detect_classification_floor(content);
    Ok(classification_floor(declared, floor))
}

/// Raise an item's classification to at least the detected floor. Never lowers.
pub fn apply_floor_to_item(item: &mut MemoryItem) {
    let floor = detect_classification_floor(&item.content);
    item.classification = classification_floor(item.classification, floor);
}

/// Validate a memory item before storage. Rejects hidden reasoning transcripts
/// heuristically (very long unstructured blobs claiming to be reasoning).
pub fn validate_item(item: &MemoryItem) -> Result<(), MemoryError> {
    if item.id.trim().is_empty() {
        return Err(MemoryError::Validation {
            reason: "Memory id must not be empty".to_string(),
        });
    }
    if item.content.trim().is_empty() {
        return Err(MemoryError::Validation {
            reason: "Memory content must not be empty".to_string(),
        });
    }
    if item.content.len() > 8000 {
        return Err(MemoryError::Validation {
            reason: "Memory content exceeds 8000 chars; store a structured fact, not a transcript"
                .to_string(),
        });
    }
    if !(0.0..=1.0).contains(&item.confidence) {
        return Err(MemoryError::Validation {
            reason: "Memory confidence must be within [0,1]".to_string(),
        });
    }
    if item.source == MemorySource::Unknown {
        return Err(MemoryError::SecurityViolation {
            reason: "Memory source UNKNOWN fails closed".to_string(),
        });
    }
    // Memory must not become system policy: reject policy-override content
    // from low-trust sources at write time.
    if !item.source.is_high_trust() {
        let lower = item.content.to_lowercase();
        for trigger in [
            "always permitted",
            "cloud access is always",
            "ignore policy",
            "override policy",
            "disable privacy",
            "upload credentials",
            "always upload",
        ] {
            if lower.contains(trigger) {
                return Err(MemoryError::SecurityViolation {
                    reason: format!("Low-trust memory attempts policy override ('{}')", trigger),
                });
            }
        }
    }
    // TemporaryFact vs durable types: no additional constraints here, but
    // ensure TaskFact/WorkspaceFact carry scope ids when scoped.
    match item.mem_type {
        MemoryType::TaskFact => {
            if item.task_id.is_none() {
                return Err(MemoryError::Validation {
                    reason: "TaskFact requires task_id scope".to_string(),
                });
            }
        }
        MemoryType::WorkspaceFact => {
            if item.workspace_id.is_none() {
                return Err(MemoryError::Validation {
                    reason: "WorkspaceFact requires workspace_id scope".to_string(),
                });
            }
        }
        _ => {}
    }
    Ok(())
}
