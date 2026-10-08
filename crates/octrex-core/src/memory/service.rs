use crate::db::manager::DatabaseManager;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::memory::errors::MemoryError;
use crate::memory::extractor::{ExtractionInput, MemoryExtractor};
use crate::memory::store::MemoryStore;
use crate::memory::types::{
    now_millis, MemoryItem, MemoryProvenance, MemoryQuery, MemoryResult, MemorySource, MemoryType,
};
use std::sync::Arc;

/// High-level memory service: user control (see/approve/reject/edit/delete),
/// scoped retrieval, and ContextEngine integration. Memory is a candidate
/// context source: it still passes Context selection + token budget + privacy
/// classification via `to_context_item`.
#[derive(Clone)]
pub struct MemoryService {
    store: MemoryStore,
    event_bus: Arc<EventBus>,
    db: Arc<DatabaseManager>,
}

impl MemoryService {
    pub fn new(db: Arc<DatabaseManager>, event_bus: Arc<EventBus>) -> Self {
        Self {
            store: MemoryStore::new(db.clone()),
            event_bus,
            db,
        }
    }

    pub fn store(&self) -> &MemoryStore {
        &self.store
    }

    fn emit(&self, event_type: EventType, payload: serde_json::Value) {
        let _ = self
            .event_bus
            .publish(EventEnvelope::new(event_type, payload));
    }

    fn audit(&self, event_type: &str, memory_id: &str, success: bool, reason: &str) {
        let id = format!("audit-{}", uuid::Uuid::new_v4().simple());
        let now = now_millis();
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_records (id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason)
                 VALUES (?1, ?2, ?3, NULL, NULL, NULL, 'memory_service', NULL, NULL, NULL, NULL, NULL, NULL, ?4, ?5, ?6)",
                rusqlite::params![id, now as i64, event_type, memory_id, if success { 1 } else { 0 }, reason],
            )
            .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
            Ok(())
        });
    }

    pub fn create_item(&self, item: MemoryItem) -> Result<MemoryItem, MemoryError> {
        let stored = self.store.create_item(item)?;
        // Never log raw secrets.
        let reason = if stored.classification == crate::privacy::PrivacyClassification::Secret {
            "created (content redacted)"
        } else {
            "created"
        };
        self.audit("MEMORY_CREATED", &stored.id, true, reason);
        Ok(stored)
    }

    pub fn update_item(
        &self,
        id: &str,
        content: Option<String>,
        confidence: Option<f64>,
    ) -> Result<MemoryItem, MemoryError> {
        let updated = self.store.update_item(id, content, confidence)?;
        self.emit(
            EventType::MemoryUpdated,
            serde_json::json!({"memory_id": id}),
        );
        self.audit("MEMORY_UPDATED", id, true, "updated");
        Ok(updated)
    }

    pub fn delete_item(&self, id: &str) -> Result<bool, MemoryError> {
        let ok = self.store.delete_item(id)?;
        self.emit(
            EventType::MemoryDeleted,
            serde_json::json!({"memory_id": id}),
        );
        self.audit("MEMORY_DELETED", id, ok, "deleted");
        Ok(ok)
    }

    pub fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryResult>, MemoryError> {
        let results = self.store.query(query)?;
        // Audit security-sensitive retrieval without raw data.
        let sensitive = results.iter().any(|r| {
            r.item.classification == crate::privacy::PrivacyClassification::Secret
                || r.item.classification == crate::privacy::PrivacyClassification::Restricted
        });
        if sensitive {
            self.audit(
                "MEMORY_RETRIEVED_SENSITIVE",
                &format!("count={}", results.len()),
                true,
                "sensitive retrieval (content not logged)",
            );
        }
        self.emit(
            EventType::MemoryRetrieved,
            serde_json::json!({"count": results.len()}),
        );
        Ok(results)
    }

    /// Extract candidates after task completion. Never auto-saves model
    /// statements as permanent memory.
    pub fn extract_candidates(
        &self,
        input: &ExtractionInput,
    ) -> Result<Vec<crate::memory::types::MemoryCandidate>, MemoryError> {
        let candidates = MemoryExtractor::extract_candidates(input)?;
        let mut stored = Vec::new();
        for c in candidates {
            let s = self.store.create_candidate(c)?;
            self.emit(
                EventType::MemoryCandidateCreated,
                serde_json::json!({"candidate_id": s.id}),
            );
            stored.push(s);
        }
        Ok(stored)
    }

    pub fn approve_candidate(&self, id: &str) -> Result<MemoryItem, MemoryError> {
        let item = self.store.approve_candidate(id)?;
        self.emit(
            EventType::MemoryApproved,
            serde_json::json!({"candidate_id": id, "memory_id": item.id}),
        );
        self.audit("MEMORY_APPROVED", id, true, "approved");
        Ok(item)
    }

    pub fn reject_candidate(
        &self,
        id: &str,
    ) -> Result<crate::memory::types::MemoryCandidate, MemoryError> {
        let cand = self.store.reject_candidate(id)?;
        self.emit(
            EventType::MemoryRejected,
            serde_json::json!({"candidate_id": id}),
        );
        self.audit("MEMORY_REJECTED", id, true, "rejected");
        Ok(cand)
    }

    pub fn list_candidates(
        &self,
        status: Option<&str>,
    ) -> Vec<crate::memory::types::MemoryCandidate> {
        self.store.list_candidates(status)
    }

    /// Convert memory results into ContextItems WITHOUT injecting directly
    /// into model prompts. Callers must pass these through ContextEngine
    /// selection + budget + classification.
    pub fn to_context_items(
        results: &[MemoryResult],
        session_id: Option<crate::ids::SessionId>,
        task_id: Option<crate::ids::TaskId>,
        workspace_id: Option<crate::ids::WorkspaceId>,
    ) -> Vec<crate::context::ContextItem> {
        results
            .iter()
            .map(|r| {
                let mut item = crate::context::ContextItem::new(
                    crate::context::ContextSource::WorkspaceContext,
                    crate::context::ContextRole::User,
                    format!(
                        "[memory:{}|{}|{}] {}",
                        r.item.mem_type, r.item.scope, r.item.source, r.item.content
                    ),
                )
                .with_classification(r.item.classification)
                .with_session(session_id.clone())
                .with_task(task_id.clone())
                .with_workspace(workspace_id.clone())
                .with_source_id(r.item.id.clone())
                .with_provenance(format!("memory:{}:{}", r.item.id, r.item.source));
                // Trust mapping: memory trust never exceeds UserControlled;
                // system/company memory maps to TrustedCompany at most, and
                // control-plane policy always outranks it in the selector.
                item.trust_level = match r.item.source {
                    MemorySource::System => crate::context::ContextTrustLevel::TrustedCompany,
                    MemorySource::Company => crate::context::ContextTrustLevel::TrustedCompany,
                    MemorySource::UserExplicit => crate::context::ContextTrustLevel::UserControlled,
                    _ => crate::context::ContextTrustLevel::ModelGenerated,
                };
                // Downgrade-proof: classification already floored at write.
                item.priority = 60;
                item
            })
            .collect()
    }

    /// Convenience constructor for direct user-controlled memory writes.
    pub fn build_user_item(
        mem_type: MemoryType,
        scope: crate::memory::types::MemoryScope,
        content: String,
        workspace_id: Option<String>,
        session_id: Option<String>,
        task_id: Option<String>,
        classification: crate::privacy::PrivacyClassification,
    ) -> MemoryItem {
        let now = now_millis();
        MemoryItem {
            id: format!("mem-{}", uuid::Uuid::new_v4().simple()),
            mem_type,
            scope,
            workspace_id,
            project_id: None,
            session_id,
            task_id,
            content,
            classification,
            source: MemorySource::UserExplicit,
            confidence: 0.9,
            provenance: MemoryProvenance {
                source: MemorySource::UserExplicit,
                actor: "user".to_string(),
                tool_id: None,
                model_id: None,
                imported_from: None,
                evidence: "explicit user statement".to_string(),
            },
            created_at: now,
            updated_at: now,
            expires_at: None,
            version: 1,
        }
    }
}
