//! Phase 15: Document Intelligence & Artifact Pipeline.
//!
//! Local-first pipeline:
//!
//! ```text
//! FilesystemSecurity
//!   -> DocumentIntake
//!   -> DocumentParser
//!   -> Classification / Security Analysis (monotonic, fail-closed)
//!   -> NormalizedDocument (provenance preserved)
//!   -> Chunker (ContextEngine-compatible, existing tokenizer)
//!   -> ContextService (FileContent / untrusted, budget-checked)
//!   -> Agent / Workflow / Skill
//!   -> Artifact Generation (UNVERIFIED by default)
//!   -> VerificationEngine (required before claiming done)
//!   -> Artifact Registry + lineage + audit
//! ```
//!
//! Invariants:
//! - Workspace isolation everywhere.
//! - Untrusted document content can never change policy/permissions/privacy,
//!   routing, capabilities, or any security decision.
//! - Classification only rises, never silently falls.
//! - No automatic local -> online/cloud fallback.
//! - No artifact auto-verification.
//! - Secrets never enter logs, audit, events, or model prompts unnecessarily.

pub mod chunker;
pub mod inflate;
pub mod intake;
pub mod parser;
pub mod pipeline;
pub mod retrieval;
pub mod service;
pub mod store;
pub mod tools;
pub mod types;
pub mod zip;

#[cfg(test)]
pub mod tests;

pub use chunker::{chunk_document, estimate_tokens, ChunkerConfig};
pub use pipeline::{
    artifact_kind_from_str, build_artifact_verification_request, validate_artifact_content,
};
pub use retrieval::retrieve_chunks;
pub use service::{DocumentService, ImportOutput};
pub use store::{
    audit_document_event, get_document_row, lineage_for_artifact, list_documents_for_workspace,
    load_chunks_for_document, save_artifact_lineage, save_chunks, save_document, save_sections,
    update_document_status,
};
pub use tools::{document_tool_descriptors, execute_document_tool};
pub use types::{
    content_hash_hex, extension_of, file_name_of, now_millis, raise_classification, ArtifactKind,
    ArtifactLineageNode, DocumentBlock, DocumentBlockKind, DocumentChunk, DocumentChunkId,
    DocumentContent, DocumentExtractionStatus, DocumentFormat, DocumentId, DocumentMetadata,
    DocumentPage, DocumentParseWarning, DocumentProvenance, DocumentRetrievalResult,
    DocumentSection, DocumentSecurityFinding, DocumentSource, DocumentVersionId,
    NormalizedDocument,
};
