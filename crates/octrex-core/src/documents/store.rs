//! SQLite persistence for Phase 15 (documents, versions, sections,
//! chunks, artifact lineage). Reuses `DatabaseManager`; migration v11 creates
//! the tables. Secrets are never stored: only references + redacted metadata.

use crate::db::DatabaseManager;
use crate::documents::types::{
    DocumentChunk, DocumentExtractionStatus, DocumentMetadata, DocumentParseWarning,
    DocumentSection, DocumentSecurityFinding,
};
use crate::error::OctrexError;
use crate::ids::WorkspaceId;
use crate::privacy::PrivacyClassification;
use rusqlite::params;

fn warnings_json(w: &[DocumentParseWarning]) -> String {
    serde_json::to_string(w).unwrap_or_else(|_| "[]".to_string())
}

fn findings_json(f: &[DocumentSecurityFinding]) -> String {
    // Findings summaries only; parser never puts secret values in summaries.
    serde_json::to_string(f).unwrap_or_else(|_| "[]".to_string())
}

pub fn save_document(db: &DatabaseManager, meta: &DocumentMetadata) -> Result<(), OctrexError> {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO documents (id, workspace_id, rel_path, file_name, format, mime, size_bytes, content_hash, classification, extraction_status, title, provenance_json, warnings_json, findings_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                meta.id.as_str(),
                meta.workspace_id.as_str(),
                meta.rel_path,
                meta.file_name,
                meta.format.as_str(),
                meta.mime,
                meta.size_bytes as i64,
                meta.content_hash,
                meta.classification.to_string(),
                meta.extraction_status.to_string(),
                meta.title,
                serde_json::to_string(&meta.provenance).unwrap_or_default(),
                warnings_json(&meta.warnings),
                findings_json(&meta.findings),
                meta.created_at as i64,
                meta.updated_at as i64,
            ],
        )
        .map_err(|e| OctrexError::Internal {
            message: format!("save_document failed: {}", e),
        })?;
        // version row (idempotent)
        conn.execute(
            "INSERT OR IGNORE INTO document_versions (id, document_id, version_number, content_hash, classification, created_at, metadata_json)
             VALUES (?1, ?2, 1, ?3, ?4, ?5, '{}')",
            params![
                meta.version_id.as_str(),
                meta.id.as_str(),
                meta.content_hash,
                meta.classification.to_string(),
                meta.created_at as i64,
            ],
        )
        .map_err(|e| OctrexError::Internal {
            message: format!("save_document_version failed: {}", e),
        })?;
        Ok(())
    })
}

pub fn save_sections(
    db: &DatabaseManager,
    document_id: &str,
    version_id: &str,
    sections: &[DocumentSection],
) -> Result<(), OctrexError> {
    db.with_conn(|conn| {
        for s in sections {
            conn.execute(
                "INSERT OR REPLACE INTO document_sections (id, document_id, version_id, title, level, line_start, line_end, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    s.id,
                    document_id,
                    version_id,
                    s.title,
                    s.level as i64,
                    s.line_start.map(|v| v as i64),
                    s.line_end.map(|v| v as i64),
                    crate::documents::types::now_millis() as i64,
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("save_section failed: {}", e),
            })?;
        }
        Ok(())
    })
}

pub fn save_chunks(db: &DatabaseManager, chunks: &[DocumentChunk]) -> Result<(), OctrexError> {
    db.with_conn(|conn| {
        for c in chunks {
            conn.execute(
                "INSERT OR REPLACE INTO document_chunks (id, document_id, version_id, chunk_index, text, char_count, token_estimate, classification, provenance_json, workspace_id, session_id, task_id, section, page, source_path, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    c.id.as_str(),
                    c.document_id.as_str(),
                    c.version_id.as_str(),
                    c.chunk_index as i64,
                    c.text,
                    c.char_count as i64,
                    c.token_estimate as i64,
                    c.classification.to_string(),
                    serde_json::to_string(&c.provenance).unwrap_or_default(),
                    c.workspace_id.as_str(),
                    c.session_id.as_ref().map(|s| s.as_str().to_string()),
                    c.task_id.as_ref().map(|t| t.as_str().to_string()),
                    c.section,
                    c.page.map(|p| p as i64),
                    c.source_path,
                    crate::documents::types::now_millis() as i64,
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("save_chunk failed: {}", e),
            })?;
        }
        Ok(())
    })
}

pub fn load_chunks_for_document(
    db: &DatabaseManager,
    document_id: &str,
    workspace_id: &WorkspaceId,
) -> Result<Vec<DocumentChunk>, OctrexError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, document_id, version_id, chunk_index, text, char_count, token_estimate, classification, provenance_json, workspace_id, session_id, task_id, section, page, source_path FROM document_chunks WHERE document_id = ?1 AND workspace_id = ?2 ORDER BY chunk_index ASC")
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let rows = stmt
            .query_map(params![document_id, workspace_id.as_str()], |row| {
                let id: String = row.get(0)?;
                let doc_id: String = row.get(1)?;
                let ver_id: String = row.get(2)?;
                let chunk_index: i64 = row.get(3)?;
                let text: String = row.get(4)?;
                let char_count: i64 = row.get(5)?;
                let token_estimate: i64 = row.get(6)?;
                let class_str: String = row.get(7)?;
                let prov_json: String = row.get(8)?;
                let ws_str: String = row.get(9)?;
                let sess: Option<String> = row.get(10)?;
                let task: Option<String> = row.get(11)?;
                let section: Option<String> = row.get(12)?;
                let page: Option<i64> = row.get(13)?;
                let source_path: String = row.get(14)?;
                Ok((
                    id, doc_id, ver_id, chunk_index, text, char_count, token_estimate, class_str,
                    prov_json, ws_str, sess, task, section, page, source_path,
                ))
            })
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let mut out = Vec::new();
        for r in rows {
            let (
                id,
                doc_id,
                ver_id,
                chunk_index,
                text,
                char_count,
                token_estimate,
                class_str,
                prov_json,
                ws_str,
                sess,
                task,
                section,
                page,
                source_path,
            ) = r.map_err(|e| OctrexError::Internal { message: e.to_string() })?;
            let provenance: crate::documents::types::DocumentProvenance =
                serde_json::from_str(&prov_json).unwrap_or(crate::documents::types::DocumentProvenance {
                    document_id: doc_id.clone(),
                    version_id: ver_id.clone(),
                    workspace_id: ws_str.clone(),
                    source_path: source_path.clone(),
                    content_hash: String::new(),
                    imported_at: 0,
                    importer_task: None,
                    importer_session: None,
                    trust: "untrusted_file".to_string(),
                });
            out.push(DocumentChunk {
                id: crate::documents::types::DocumentChunkId(id),
                document_id: crate::documents::types::DocumentId(doc_id),
                version_id: crate::documents::types::DocumentVersionId(ver_id),
                chunk_index: chunk_index as u32,
                text,
                char_count: char_count as usize,
                token_estimate: token_estimate as usize,
                token_kind: "estimated".to_string(),
                classification: class_str
                    .parse::<PrivacyClassification>()
                    .unwrap_or(PrivacyClassification::Public),
                provenance,
                workspace_id: WorkspaceId::from(ws_str),
                session_id: sess.map(crate::ids::SessionId::from),
                task_id: task.map(crate::ids::TaskId::from),
                section,
                page: page.map(|p| p as u32),
                source_path,
            });
        }
        Ok(out)
    })
}

pub fn list_documents_for_workspace(
    db: &DatabaseManager,
    workspace_id: &WorkspaceId,
) -> Result<Vec<serde_json::Value>, OctrexError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, rel_path, file_name, format, mime, size_bytes, content_hash, classification, extraction_status, title, created_at FROM documents WHERE workspace_id = ?1 ORDER BY created_at DESC LIMIT 200")
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let rows = stmt
            .query_map(params![workspace_id.as_str()], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "rel_path": row.get::<_, String>(1)?,
                    "file_name": row.get::<_, String>(2)?,
                    "format": row.get::<_, String>(3)?,
                    "mime": row.get::<_, String>(4)?,
                    "size_bytes": row.get::<_, i64>(5)?,
                    "content_hash": row.get::<_, String>(6)?,
                    "classification": row.get::<_, String>(7)?,
                    "extraction_status": row.get::<_, String>(8)?,
                    "title": row.get::<_, Option<String>>(9)?,
                    "created_at": row.get::<_, i64>(10)?,
                }))
            })
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
        }
        Ok(out)
    })
}

pub fn get_document_row(
    db: &DatabaseManager,
    document_id: &str,
) -> Result<Option<serde_json::Value>, OctrexError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, workspace_id, rel_path, file_name, format, mime, size_bytes, content_hash, classification, extraction_status, title, provenance_json, warnings_json, findings_json, created_at FROM documents WHERE id = ?1")
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let res: Result<serde_json::Value, rusqlite::Error> =
            stmt.query_row(params![document_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "workspace_id": row.get::<_, String>(1)?,
                    "rel_path": row.get::<_, String>(2)?,
                    "file_name": row.get::<_, String>(3)?,
                    "format": row.get::<_, String>(4)?,
                    "mime": row.get::<_, String>(5)?,
                    "size_bytes": row.get::<_, i64>(6)?,
                    "content_hash": row.get::<_, String>(7)?,
                    "classification": row.get::<_, String>(8)?,
                    "extraction_status": row.get::<_, String>(9)?,
                    "title": row.get::<_, Option<String>>(10)?,
                    "provenance": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(11)?).unwrap_or(serde_json::Value::Null),
                    "warnings": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(12)?).unwrap_or(serde_json::Value::Array(vec![])),
                    "findings": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(13)?).unwrap_or(serde_json::Value::Array(vec![])),
                    "created_at": row.get::<_, i64>(14)?,
                }))
            });
        match res {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub fn save_artifact_lineage(
    db: &DatabaseManager,
    artifact_id: &str,
    parent_artifact_id: Option<&str>,
    source_document_id: Option<&str>,
    source_document_version: Option<&str>,
    producing_workflow: Option<&str>,
    producing_skill: Option<&str>,
    producing_model: Option<&str>,
    producing_provider: Option<&str>,
    classification: &PrivacyClassification,
    metadata_json: &str,
) -> Result<String, OctrexError> {
    let lineage_id = format!("lin-{}", uuid::Uuid::new_v4().simple());
    let now = crate::documents::types::now_millis() as i64;
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO artifact_lineage (id, artifact_id, parent_artifact_id, source_document_id, source_document_version, producing_workflow, producing_skill, producing_model, producing_provider, classification, created_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                lineage_id,
                artifact_id,
                parent_artifact_id,
                source_document_id,
                source_document_version,
                producing_workflow,
                producing_skill,
                producing_model,
                producing_provider,
                classification.to_string(),
                now,
                metadata_json,
            ],
        )
        .map_err(|e| OctrexError::Internal {
            message: format!("save_artifact_lineage failed: {}", e),
        })?;
        Ok(lineage_id.clone())
    })
}

pub fn lineage_for_artifact(
    db: &DatabaseManager,
    artifact_id: &str,
) -> Result<Vec<serde_json::Value>, OctrexError> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, artifact_id, parent_artifact_id, source_document_id, source_document_version, producing_workflow, producing_skill, producing_model, producing_provider, classification, created_at FROM artifact_lineage WHERE artifact_id = ?1 ORDER BY created_at ASC")
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let rows = stmt
            .query_map(params![artifact_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "artifact_id": row.get::<_, String>(1)?,
                    "parent_artifact_id": row.get::<_, Option<String>>(2)?,
                    "source_document_id": row.get::<_, Option<String>>(3)?,
                    "source_document_version": row.get::<_, Option<String>>(4)?,
                    "producing_workflow": row.get::<_, Option<String>>(5)?,
                    "producing_skill": row.get::<_, Option<String>>(6)?,
                    "producing_model": row.get::<_, Option<String>>(7)?,
                    "producing_provider": row.get::<_, Option<String>>(8)?,
                    "classification": row.get::<_, String>(9)?,
                    "created_at": row.get::<_, i64>(10)?,
                }))
            })
            .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
        }
        Ok(out)
    })
}

#[allow(clippy::too_many_arguments)]
pub fn audit_document_event(
    db: &DatabaseManager,
    event_type: &str,
    workspace_id: Option<&str>,
    task_id: Option<&str>,
    session_id: Option<&str>,
    classification: &PrivacyClassification,
    success: bool,
    reason: &str,
) {
    let id = format!("audit-{}", uuid::Uuid::new_v4().simple());
    let now = crate::documents::types::now_millis() as i64;
    // Never store secret content: reason must already be redacted by caller.
    let _ = db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO audit_records (id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'document_service', NULL, NULL, NULL, ?7, 'document_policy', NULL, NULL, ?8, ?9)",
            params![
                id,
                now,
                event_type,
                task_id,
                session_id,
                workspace_id,
                classification.to_string(),
                if success { 1 } else { 0 },
                reason,
            ],
        )
        .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        Ok::<(), OctrexError>(())
    });
}

/// Best-effort status update mirroring extraction outcome.
pub fn update_document_status(
    db: &DatabaseManager,
    document_id: &str,
    status: &DocumentExtractionStatus,
    classification: &PrivacyClassification,
    title: Option<&str>,
    warnings: &[DocumentParseWarning],
    findings: &[DocumentSecurityFinding],
) {
    let _ = db.with_conn(|conn| {
        conn.execute(
            "UPDATE documents SET extraction_status = ?1, classification = ?2, title = COALESCE(?3, title), warnings_json = ?4, findings_json = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                status.to_string(),
                classification.to_string(),
                title,
                warnings_json(warnings),
                findings_json(findings),
                crate::documents::types::now_millis() as i64,
                document_id,
            ],
        )
        .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
        Ok::<(), OctrexError>(())
    });
}
