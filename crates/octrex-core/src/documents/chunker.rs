//! Document chunking into ContextEngine-compatible chunks.
//!
//! Uses the existing tokenizer abstraction (`DefaultTokenizer`, ~4 chars per
//! token with the ContextEngine 1.15 safety factor). No second budget
//! algorithm: callers validate against `TokenBudget` via ContextService.

use crate::documents::types::{DocumentChunk, DocumentChunkId, NormalizedDocument};
use crate::ids::{SessionId, TaskId};

#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub max_chars_per_chunk: usize,
    pub overlap_chars: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_chars_per_chunk: 1500,
            overlap_chars: 200,
        }
    }
}

/// ~4 chars per token with 1.15 safety factor (mirrors ContextTokenCounter).
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let base = text.chars().count().div_ceil(4);
    ((base as f32) * 1.15).ceil() as usize
}

pub fn chunk_document(
    doc: &NormalizedDocument,
    config: &ChunkerConfig,
    session_id: Option<SessionId>,
    task_id: Option<TaskId>,
) -> Vec<DocumentChunk> {
    let mut out = Vec::new();
    let mut idx: u32 = 0;
    // Chunk each block independently to preserve section/page provenance;
    // split oversized blocks with overlap.
    for block in &doc.blocks {
        let text = block.text.clone();
        if text.chars().count() <= config.max_chars_per_chunk {
            out.push(DocumentChunk {
                id: DocumentChunkId::new(),
                document_id: doc.document_id.clone(),
                version_id: doc.version_id.clone(),
                chunk_index: idx,
                char_count: text.chars().count(),
                token_estimate: estimate_tokens(&text),
                token_kind: "estimated".to_string(),
                classification: doc.classification,
                provenance: doc.provenance.clone(),
                workspace_id: doc.source.workspace_id.clone(),
                session_id: session_id.clone(),
                task_id: task_id.clone(),
                section: block.section_id.clone(),
                page: block.page,
                source_path: block.source_ref.clone(),
                text,
                // id set above; keep struct order
            });
            idx += 1;
        } else {
            let chars: Vec<char> = text.chars().collect();
            let mut start = 0usize;
            while start < chars.len() {
                let end = (start + config.max_chars_per_chunk).min(chars.len());
                let slice: String = chars[start..end].iter().collect();
                out.push(DocumentChunk {
                    id: DocumentChunkId::new(),
                    document_id: doc.document_id.clone(),
                    version_id: doc.version_id.clone(),
                    chunk_index: idx,
                    char_count: slice.chars().count(),
                    token_estimate: estimate_tokens(&slice),
                    token_kind: "estimated".to_string(),
                    classification: doc.classification,
                    provenance: doc.provenance.clone(),
                    workspace_id: doc.source.workspace_id.clone(),
                    session_id: session_id.clone(),
                    task_id: task_id.clone(),
                    section: block.section_id.clone(),
                    page: block.page,
                    source_path: format!("{}[chars {}-{}]", block.source_ref, start, end),
                    text: slice,
                });
                idx += 1;
                if end == chars.len() {
                    break;
                }
                start = end.saturating_sub(config.overlap_chars);
                if out.len() > 10_000 {
                    break;
                }
            }
        }
        if out.len() > 10_000 {
            break;
        }
    }
    // Empty document -> single empty marker? No: return empty vec (caller handles).
    out
}
