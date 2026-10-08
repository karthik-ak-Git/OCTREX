//! Local-first deterministic lexical retrieval over document chunks.
//!
//! No second retrieval architecture: if a vector index exists in the future,
//! it can rank the same chunk records. Retrieval enforces workspace isolation
//! and the caller's classification ceiling.

use crate::documents::types::{DocumentChunk, DocumentRetrievalResult};
use crate::ids::WorkspaceId;
use crate::privacy::PrivacyClassification;

fn tokenize_query(q: &str) -> Vec<String> {
    q.split(|c: char| !c.is_alphanumeric())
        .map(|s| s.to_lowercase())
        .filter(|s| s.len() >= 2)
        .collect()
}

fn score_chunk(terms: &[String], text: &str) -> f64 {
    if terms.is_empty() {
        return 0.0;
    }
    let lower = text.to_lowercase();
    let mut hits = 0.0;
    for t in terms {
        // count occurrences bounded
        let mut count = 0usize;
        let mut from = 0usize;
        while let Some(p) = lower[from..].find(t.as_str()) {
            count += 1;
            from += p + t.len();
            if count >= 20 {
                break;
            }
        }
        if count > 0 {
            hits += 1.0 + (count as f64).ln();
        }
    }
    hits / (terms.len() as f64)
}

pub fn retrieve_chunks(
    chunks: &[DocumentChunk],
    query: &str,
    workspace_id: &WorkspaceId,
    ceiling: PrivacyClassification,
    limit: usize,
) -> Vec<DocumentRetrievalResult> {
    let terms = tokenize_query(query);
    let mut scored: Vec<(f64, &DocumentChunk)> = Vec::new();
    for c in chunks {
        // Workspace isolation: never leak across workspaces.
        if &c.workspace_id != workspace_id {
            continue;
        }
        // Classification ceiling: chunk must be at or below caller's ceiling.
        if c.classification > ceiling {
            continue;
        }
        let mut s = score_chunk(&terms, &c.text);
        if query.trim().is_empty() {
            s = 0.1;
        }
        // Boost exact phrase
        if !query.trim().is_empty() && c.text.to_lowercase().contains(&query.to_lowercase()) {
            s += 0.5;
        }
        if s > 0.0 {
            scored.push((s, c));
        }
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
        .into_iter()
        .take(limit.clamp(1, 50))
        .map(|(s, c)| DocumentRetrievalResult {
            chunk_id: c.id.as_str().to_string(),
            document_id: c.document_id.as_str().to_string(),
            version_id: c.version_id.as_str().to_string(),
            relevance: (s * 100.0).round() / 100.0,
            text: c.text.clone(),
            source: c.source_path.clone(),
            page: c.page,
            section: c.section.clone(),
            classification: c.classification,
            provenance: c.provenance.clone(),
        })
        .collect()
}
