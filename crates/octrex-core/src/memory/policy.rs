use crate::memory::errors::MemoryError;
use crate::memory::types::{MemoryItem, MemoryQuery};
use crate::privacy::PrivacyClassification;

/// Memory policy: scope isolation, classification ceiling, secret protection,
/// and the invariant that memory never outranks system/company/security/
/// privacy/permission policy.
pub struct MemoryPolicy;

impl MemoryPolicy {
    pub fn check_create(item: &MemoryItem) -> Result<(), MemoryError> {
        crate::memory::classifier::validate_item(item)?;
        Ok(())
    }

    pub fn check_query(query: &MemoryQuery) -> Result<(), MemoryError> {
        if query.limit == 0 || query.limit > 50 {
            return Err(MemoryError::Validation {
                reason: "Memory query limit must be within [1,50]".to_string(),
            });
        }
        Ok(())
    }

    /// Memory never outranks policy: model-derived memory cannot become a
    /// Constraint/Procedure with high confidence, and SECRET memory is never
    /// returned to online/model callers without explicit clearance.
    pub fn check_retrieval_for_online_model(
        item: &MemoryItem,
        is_online: bool,
    ) -> Result<(), MemoryError> {
        if is_online
            && (item.classification == PrivacyClassification::Secret
                || item.classification == PrivacyClassification::Restricted)
        {
            return Err(MemoryError::AccessDenied {
                reason: "Online model callers cannot receive SECRET/RESTRICTED memory without explicit clearance"
                    .to_string(),
            });
        }
        Ok(())
    }
}

/// Safe provenance summary: never expose raw secrets.
pub fn safe_memory_summary(item: &MemoryItem) -> serde_json::Value {
    let content = if item.classification == PrivacyClassification::Secret {
        "[REDACTED: SECRET memory]".to_string()
    } else if item.classification == PrivacyClassification::Restricted {
        "[RESTRICTED: content withheld]".to_string()
    } else if item.content.len() > 160 {
        format!("{}...", &item.content[..160])
    } else {
        item.content.clone()
    };
    serde_json::json!({
        "id": item.id,
        "type": item.mem_type.to_string(),
        "scope": item.scope.to_string(),
        "classification": item.classification.to_string(),
        "source": item.source.to_string(),
        "confidence": item.confidence,
        "content_preview": content,
        "expires_at": item.expires_at,
        "version": item.version,
    })
}
