use crate::memory::errors::MemoryError;
use crate::memory::types::{MemoryItem, MemoryQuery, MemoryResult, MemoryScope};

/// Scope isolation + classification ceiling + trust gate. Expired items are
/// never returned.
pub fn filter_items(items: &[MemoryItem], query: &MemoryQuery, now: u64) -> Vec<MemoryItem> {
    items
        .iter()
        .filter(|item| {
            if let Some(exp) = item.expires_at {
                if now > exp {
                    return false;
                }
            }
            // Classification ceiling: item must be at or below caller's ceiling.
            if (item.classification as u8) > (query.classification_ceiling as u8) {
                return false;
            }
            // Secret requires explicit opt-in.
            if item.classification == crate::privacy::PrivacyClassification::Secret
                && !query.allow_secret
            {
                return false;
            }
            // Trust gate.
            if item.source.trust_rank() < query.min_trust_rank {
                return false;
            }
            // Type filter.
            if let Some(t) = query.mem_type {
                if item.mem_type != t {
                    return false;
                }
            }
            // Scope filter (strict isolation).
            if !scope_visible(item, query) {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

fn scope_visible(item: &MemoryItem, query: &MemoryQuery) -> bool {
    // Explicit scope filter first.
    if let Some(scope) = &query.scope {
        if &item.scope != scope {
            return false;
        }
    }
    match item.scope {
        MemoryScope::Global => true,
        MemoryScope::Workspace => match (&item.workspace_id, &query.workspace_id) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
        MemoryScope::Project => match (&item.project_id, &query.project_id) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
        MemoryScope::Session => match (&item.session_id, &query.session_id) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
        MemoryScope::Task => match (&item.task_id, &query.task_id) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
    }
}

/// Rank by relevance + recency + trust. Never retrieves everything into every
/// context; callers must pass a bounded limit.
pub fn rank_items(items: Vec<MemoryItem>, query: &MemoryQuery, now: u64) -> Vec<MemoryResult> {
    let mut scored: Vec<MemoryResult> = items
        .into_iter()
        .map(|item| {
            let mut relevance = 0.0;
            // Trust contributes.
            relevance += f64::from(item.source.trust_rank()) / 100.0 * 30.0;
            // Confidence contributes.
            relevance += item.confidence * 30.0;
            // Recency: newer is better (decay over 30 days).
            let age_ms = now.saturating_sub(item.updated_at) as f64;
            let recency = (-age_ms / (30.0 * 24.0 * 3600.0 * 1000.0)).exp();
            relevance += recency * 20.0;
            // Text overlap.
            if let Some(q) = &query.query_text {
                let ql = q.to_lowercase();
                let cl = item.content.to_lowercase();
                let tokens: Vec<&str> = ql
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|t| t.len() > 2)
                    .collect();
                if !tokens.is_empty() {
                    let hits = tokens.iter().filter(|t| cl.contains(**t)).count();
                    relevance += (hits as f64 / tokens.len() as f64) * 20.0;
                }
            }
            MemoryResult { item, relevance }
        })
        .collect();
    scored.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(query.limit.max(1).min(50));
    scored
}

pub fn check_access(item: &MemoryItem, query: &MemoryQuery) -> Result<(), MemoryError> {
    if item.classification == crate::privacy::PrivacyClassification::Secret && !query.allow_secret {
        return Err(MemoryError::AccessDenied {
            reason: "SECRET memory requires explicit allow_secret".to_string(),
        });
    }
    if !scope_visible(item, query) {
        return Err(MemoryError::AccessDenied {
            reason: "Memory scope isolation denied access".to_string(),
        });
    }
    Ok(())
}
