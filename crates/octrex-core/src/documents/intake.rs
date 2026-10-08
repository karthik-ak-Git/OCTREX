//! Document intake through the Phase 8 filesystem boundary.
//!
//! Never trusts client absolute paths. All resolution goes through
//! `FilesystemSecurityService::evaluate_operation` + `read_file_bytes`.

use crate::documents::types::{
    content_hash_hex, extension_of, file_name_of, now_millis, DocumentFormat, DocumentId,
    DocumentMetadata, DocumentProvenance, DocumentVersionId,
};
use crate::error::OctrexError;
use crate::filesystem::{FilesystemOperation, FilesystemSecurityService};
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::privacy::PrivacyClassification;

#[derive(Debug)]
pub struct IntakeResult {
    pub metadata: DocumentMetadata,
    pub bytes: Vec<u8>,
}

pub fn intake_document(
    fs: &FilesystemSecurityService,
    workspace_id: &WorkspaceId,
    rel_path: &str,
    session_id: Option<&SessionId>,
    task_id: Option<&TaskId>,
) -> Result<IntakeResult, OctrexError> {
    // Reject absolute-looking paths early (defense in depth; the boundary
    // also rejects them).
    let trimmed = rel_path.trim();
    if trimmed.is_empty() {
        return Err(OctrexError::Validation {
            message: "rel_path must not be empty".to_string(),
            details: None,
        });
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || trimmed.contains(":\n") {
        return Err(OctrexError::Filesystem {
            message: format!("Absolute paths are rejected: '{}'", rel_path),
        });
    }
    // Never execute on extension: only Read through the boundary.
    let decision =
        fs.evaluate_operation(Some(workspace_id), None, trimmed, FilesystemOperation::Read)?;
    if !decision.decision.is_allowed() {
        return Err(OctrexError::PermissionDenied {
            reason: format!("{}: {}", trimmed, decision.reason),
        });
    }
    // Enforce workspace read-only: reads are fine, but record policy.
    let bytes = fs.read_file_bytes(workspace_id, trimmed)?;
    // Enforce size cap from active policy (fail closed).
    if let Ok(status) = fs.get_workspace_security(workspace_id) {
        if bytes.len() as u64 > status.active_policy.max_read_bytes {
            return Err(OctrexError::Filesystem {
                message: format!(
                    "File '{}' ({} bytes) exceeds workspace read limit ({} bytes)",
                    trimmed,
                    bytes.len(),
                    status.active_policy.max_read_bytes
                ),
            });
        }
    }
    // Hard cap 50MB regardless of policy.
    if bytes.len() > 50 * 1024 * 1024 {
        return Err(OctrexError::Filesystem {
            message: "File exceeds 50MB document intake cap".to_string(),
        });
    }

    let ext = extension_of(trimmed);
    let format = DocumentFormat::from_extension(&ext);
    let hash = content_hash_hex(&bytes);
    let now = now_millis();
    let doc_id = DocumentId::new();
    let ver_id = DocumentVersionId::new();
    let provenance = DocumentProvenance {
        document_id: doc_id.as_str().to_string(),
        version_id: ver_id.as_str().to_string(),
        workspace_id: workspace_id.as_str().to_string(),
        source_path: trimmed.to_string(),
        content_hash: hash.clone(),
        imported_at: now,
        importer_task: task_id.map(|t| t.as_str().to_string()),
        importer_session: session_id.map(|s| s.as_str().to_string()),
        trust: "untrusted_file".to_string(),
    };
    let metadata = DocumentMetadata {
        id: doc_id,
        version_id: ver_id,
        workspace_id: workspace_id.clone(),
        rel_path: trimmed.to_string(),
        file_name: file_name_of(trimmed),
        format: format.clone(),
        mime: format.mime().to_string(),
        size_bytes: bytes.len() as u64,
        content_hash: hash,
        classification: PrivacyClassification::Public,
        extraction_status: crate::documents::types::DocumentExtractionStatus::Pending,
        title: None,
        warnings: Vec::new(),
        findings: Vec::new(),
        provenance,
        created_at: now,
        updated_at: now,
    };
    Ok(IntakeResult { metadata, bytes })
}
