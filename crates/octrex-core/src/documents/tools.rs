//! Document ToolRuntime descriptors + filesystem-bound executors.
//!
//! Every tool declares capabilities and goes through the existing
//! `ToolRuntime` policy, privacy, network, consent, audit, and output
//! sanitization pipeline. Unknown capability/tool → DENY (enforced by
//! `ToolRuntime`, not here).

use crate::documents::parser::{parse_bytes, scan_text_for_findings};
use crate::documents::types::{content_hash_hex, extension_of, DocumentFormat};
use crate::privacy::{EvidenceManager, PrivacyClassification, PrivacyClassifier, PrivacyInput};
use crate::tools::errors::ToolError;
use crate::tools::sandbox::ToolExecutionContext;
use crate::tools::types::{RiskLevel, ToolCapability, ToolDescriptor, ToolId, ToolSource};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn document_tool_descriptors() -> Vec<ToolDescriptor> {
    let mut docs = Vec::new();
    let mut meta = HashMap::new();
    meta.insert("phase".to_string(), "15".to_string());

    let defs: Vec<(&str, &str, Vec<ToolCapability>, RiskLevel, serde_json::Value)> = vec![
        (
            "document.inspect",
            "Inspect document metadata (format, size, hash, classification) without full extraction",
            vec![ToolCapability::FilesystemRead, ToolCapability::WorkspaceRead],
            RiskLevel::Low,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        ),
        (
            "document.extract",
            "Extract text/content from a supported document with provenance",
            vec![ToolCapability::FilesystemRead, ToolCapability::WorkspaceRead],
            RiskLevel::Low,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"max_chars":{"type":"number"}},"required":["path"]}),
        ),
        (
            "document.search",
            "Lexical search inside a single document (workspace-isolated)",
            vec![ToolCapability::FilesystemRead, ToolCapability::WorkspaceRead],
            RiskLevel::Low,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"query":{"type":"string"}},"required":["path","query"]}),
        ),
        (
            "document.read_section",
            "Read one section/heading from a markdown document",
            vec![ToolCapability::FilesystemRead, ToolCapability::WorkspaceRead],
            RiskLevel::Low,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"section":{"type":"string"}},"required":["path","section"]}),
        ),
        (
            "document.list_sections",
            "List sections/headings/pages of a document",
            vec![ToolCapability::FilesystemRead, ToolCapability::WorkspaceRead],
            RiskLevel::Low,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        ),
        (
            "document.chunk",
            "Chunk a document into ContextEngine-compatible chunk previews",
            vec![ToolCapability::FilesystemRead, ToolCapability::WorkspaceRead],
            RiskLevel::Low,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"max_chars":{"type":"number"}},"required":["path"]}),
        ),
        (
            "document.create_artifact",
            "Create a derived artifact file inside the workspace (UNVERIFIED until VerificationEngine passes)",
            vec![ToolCapability::FilesystemWrite, ToolCapability::WorkspaceWrite],
            RiskLevel::Medium,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"},"artifact_type":{"type":"string"}},"required":["path","content"]}),
        ),
        (
            "document.export_artifact",
            "Copy an artifact within the workspace (external export requires the authorized server export API)",
            vec![ToolCapability::FilesystemExport, ToolCapability::WorkspaceExport],
            RiskLevel::High,
            serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"dest":{"type":"string"}},"required":["path","dest"]}),
        ),
    ];

    for (id, desc, caps, risk, schema) in defs {
        docs.push(ToolDescriptor {
            id: ToolId::new(id),
            name: desc.to_string(),
            version: "15.0.0".to_string(),
            description: desc.to_string(),
            source: ToolSource::BuiltIn,
            capabilities: caps,
            input_schema: schema,
            output_schema: None,
            risk_level: risk,
            enabled: true,
            requires_confirmation: id == "document.export_artifact",
            timeout_ms: 20_000,
            metadata: meta.clone(),
        });
    }
    docs
}

fn require_cap(
    ctx: &ToolExecutionContext,
    cap: &ToolCapability,
    alt: &ToolCapability,
) -> Result<(), ToolError> {
    if ctx.has_capability(cap) || ctx.has_capability(alt) {
        Ok(())
    } else {
        Err(ToolError::CapabilityDenied {
            capability: cap.to_string(),
            reason: "Missing required document capability".to_string(),
        })
    }
}

fn resolve(ctx: &ToolExecutionContext, rel: &str) -> Result<PathBuf, ToolError> {
    let ws_root = ctx.workspace_path.as_ref().ok_or_else(|| {
        ToolError::FilesystemDenied("No active workspace path bound to context".to_string())
    })?;
    let clean = rel.trim_start_matches('/').trim_start_matches('\\');
    if clean.contains("..") {
        return Err(ToolError::FilesystemDenied(format!(
            "Path '{}' contains parent references",
            rel
        )));
    }
    let target = ws_root.join(clean);
    if let (Ok(root), Ok(t)) = (ws_root.canonicalize(), target.canonicalize()) {
        if !t.starts_with(&root) {
            return Err(ToolError::FilesystemDenied(format!(
                "Path '{}' escapes workspace boundary",
                rel
            )));
        }
        Ok(t)
    } else {
        // Not yet existing (create path): ensure parent stays inside root lexically.
        let mut cur = target.clone();
        // walk up until existing ancestor
        while !cur.exists() {
            match cur.parent() {
                Some(p) => cur = p.to_path_buf(),
                None => break,
            }
        }
        if let (Ok(root), Ok(anc)) = (ws_root.canonicalize(), cur.canonicalize()) {
            if !anc.starts_with(&root) {
                return Err(ToolError::FilesystemDenied(format!(
                    "Path '{}' escapes workspace boundary",
                    rel
                )));
            }
        }
        Ok(target)
    }
}

fn classify_text(text: &str, origin: &str) -> PrivacyClassification {
    let c = PrivacyClassifier::new();
    let input = PrivacyInput::new(
        crate::privacy::InputSourceType::FileContent,
        text.to_string(),
    )
    .with_origin(origin.to_string());
    c.classify(PrivacyClassification::Public, &[input])
        .classification
}

fn redacted_preview(text: &str, class: PrivacyClassification, max: usize) -> String {
    if class >= PrivacyClassification::Secret {
        "[REDACTED: SECRET document content withheld from tool output]".to_string()
    } else if class >= PrivacyClassification::Restricted {
        let safe: String = text.chars().take(max.min(500)).collect();
        format!(
            "[RESTRICTED preview] {}",
            EvidenceManager::redact_string(&safe)
        )
    } else {
        let preview: String = text.chars().take(max).collect();
        EvidenceManager::redact_string(&preview)
    }
}

pub fn execute_document_tool(
    tool_id: &str,
    args: &serde_json::Value,
    ctx: &ToolExecutionContext,
) -> Result<serde_json::Value, ToolError> {
    match tool_id {
        "document.inspect" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemRead,
                &ToolCapability::WorkspaceRead,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let target = resolve(ctx, p)?;
            let meta = fs::metadata(&target)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("stat failed: {}", e)))?;
            if meta.is_dir() {
                return Err(ToolError::InvalidArguments(
                    "Path is a directory".to_string(),
                ));
            }
            let bytes = fs::read(&target)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("read failed: {}", e)))?;
            let ext = extension_of(p);
            let fmt = DocumentFormat::from_extension(&ext);
            let class = if bytes.len() < 2_000_000 {
                classify_text(&String::from_utf8_lossy(&bytes), p)
            } else {
                PrivacyClassification::Public
            };
            Ok(serde_json::json!({
                "path": p,
                "file_name": p.replace('\\', "/").rsplit('/').next().unwrap_or(p),
                "extension": ext,
                "format": fmt.as_str(),
                "supported": fmt.is_supported(),
                "mime": fmt.mime(),
                "size_bytes": bytes.len(),
                "content_hash": content_hash_hex(&bytes),
                "classification": class.to_string(),
                "trust": "untrusted_file",
            }))
        }
        "document.extract" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemRead,
                &ToolCapability::WorkspaceRead,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let max_chars = args
                .get("max_chars")
                .and_then(|v| v.as_u64())
                .unwrap_or(20_000) as usize;
            let target = resolve(ctx, p)?;
            let bytes = fs::read(&target)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("read failed: {}", e)))?;
            if bytes.len() > 50 * 1024 * 1024 {
                return Err(ToolError::ToolExecutionFailed(
                    "File exceeds 50MB tool cap".to_string(),
                ));
            }
            let ext = extension_of(p);
            let fmt = DocumentFormat::from_extension(&ext);
            if !fmt.is_supported() {
                return Ok(serde_json::json!({
                    "path": p,
                    "status": "UNSUPPORTED_FORMAT",
                    "reason": format!("Extension '.{}' is not supported; refusing to guess", ext),
                }));
            }
            match parse_bytes(&fmt, &bytes, p) {
                crate::documents::parser::ParseOutcome::Parsed(parsed) => {
                    let class = classify_text(&parsed.full_text, p);
                    let findings = scan_text_for_findings(&parsed.full_text, Some(p.to_string()));
                    Ok(serde_json::json!({
                        "path": p,
                        "status": "COMPLETED",
                        "format": fmt.as_str(),
                        "title": parsed.title,
                        "blocks": parsed.blocks.len(),
                        "sections": parsed.sections.len(),
                        "pages": parsed.pages.len(),
                        "classification": class.to_string(),
                        "content_hash": content_hash_hex(&bytes),
                        "warnings": parsed.warnings,
                        "security_findings": findings,
                        "text_preview": redacted_preview(&parsed.full_text, class, max_chars.min(20_000)),
                        "total_chars": parsed.full_text.len(),
                    }))
                }
                crate::documents::parser::ParseOutcome::UnsupportedFormat { extension, reason } => {
                    Ok(
                        serde_json::json!({"path": p, "status": "UNSUPPORTED_FORMAT", "extension": extension, "reason": reason}),
                    )
                }
                crate::documents::parser::ParseOutcome::Failed { reason, warnings } => Ok(
                    serde_json::json!({"path": p, "status": "FAILED", "reason": reason, "warnings": warnings}),
                ),
            }
        }
        "document.search" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemRead,
                &ToolCapability::WorkspaceRead,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let q = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'query'".to_string()))?;
            if q.trim().len() < 2 {
                return Err(ToolError::InvalidArguments("query too short".to_string()));
            }
            let target = resolve(ctx, p)?;
            let content = fs::read_to_string(&target).map_err(|_| {
                ToolError::ToolExecutionFailed(
                    "File is not UTF-8 text or is binary; use document.extract for format-aware parsing".to_string(),
                )
            })?;
            let class = classify_text(&content, p);
            let mut matches = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&q.to_lowercase()) {
                    matches.push(serde_json::json!({
                        "line_number": i + 1,
                        "line_text": EvidenceManager::redact_string(line.trim()),
                    }));
                    if matches.len() >= 50 {
                        break;
                    }
                }
            }
            Ok(serde_json::json!({
                "path": p,
                "query": q,
                "match_count": matches.len(),
                "classification": class.to_string(),
                "matches": if class >= PrivacyClassification::Secret { serde_json::Value::Array(vec![]) } else { serde_json::Value::Array(matches) },
                "note": if class >= PrivacyClassification::Secret { "SECRET content: match locations withheld" } else { "" },
            }))
        }
        "document.list_sections" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemRead,
                &ToolCapability::WorkspaceRead,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let target = resolve(ctx, p)?;
            let bytes = fs::read(&target)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("read failed: {}", e)))?;
            let ext = extension_of(p);
            let fmt = DocumentFormat::from_extension(&ext);
            if !fmt.is_supported() {
                return Ok(serde_json::json!({"path": p, "status": "UNSUPPORTED_FORMAT"}));
            }
            match parse_bytes(&fmt, &bytes, p) {
                crate::documents::parser::ParseOutcome::Parsed(parsed) => Ok(serde_json::json!({
                    "path": p,
                    "sections": parsed.sections,
                    "pages": parsed.pages.iter().map(|pg| serde_json::json!({"number": pg.number, "chars": pg.text.len()})).collect::<Vec<_>>(),
                    "blocks": parsed.blocks.len(),
                })),
                other => Ok(
                    serde_json::json!({"path": p, "status": format!("{:?}", std::mem::discriminant(&other))}),
                ),
            }
        }
        "document.read_section" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemRead,
                &ToolCapability::WorkspaceRead,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let sec = args
                .get("section")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'section'".to_string()))?;
            let target = resolve(ctx, p)?;
            let bytes = fs::read(&target)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("read failed: {}", e)))?;
            let ext = extension_of(p);
            let fmt = DocumentFormat::from_extension(&ext);
            match parse_bytes(&fmt, &bytes, p) {
                crate::documents::parser::ParseOutcome::Parsed(parsed) => {
                    let class = classify_text(&parsed.full_text, p);
                    // match by section id or title substring
                    let needle = sec.to_lowercase();
                    let matched: Vec<_> = parsed
                        .blocks
                        .iter()
                        .filter(|b| {
                            b.section_id.as_deref() == Some(sec)
                                || parsed.sections.iter().any(|s| {
                                    Some(&s.id) == b.section_id.as_ref()
                                        && s.title.to_lowercase().contains(&needle)
                                })
                        })
                        .collect();
                    let joined = matched
                        .iter()
                        .map(|b| b.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    Ok(serde_json::json!({
                        "path": p,
                        "section": sec,
                        "blocks": matched.len(),
                        "classification": class.to_string(),
                        "text": redacted_preview(&joined, class, 10_000),
                    }))
                }
                _ => Ok(serde_json::json!({"path": p, "section": sec, "status": "UNPARSEABLE"})),
            }
        }
        "document.chunk" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemRead,
                &ToolCapability::WorkspaceRead,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let max_chars = args
                .get("max_chars")
                .and_then(|v| v.as_u64())
                .unwrap_or(1500) as usize;
            let target = resolve(ctx, p)?;
            let bytes = fs::read(&target)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("read failed: {}", e)))?;
            let ext = extension_of(p);
            let fmt = DocumentFormat::from_extension(&ext);
            match parse_bytes(&fmt, &bytes, p) {
                crate::documents::parser::ParseOutcome::Parsed(parsed) => {
                    let class = classify_text(&parsed.full_text, p);
                    // naive chunk previews (tool-scoped; full chunking lives in DocumentService)
                    let mut chunks = Vec::new();
                    let mut buf = String::new();
                    let mut idx = 0;
                    for b in &parsed.blocks {
                        if buf.len() + b.text.len() + 1 > max_chars.clamp(500, 8000)
                            && !buf.is_empty()
                        {
                            chunks.push(serde_json::json!({
                                "chunk_index": idx,
                                "chars": buf.len(),
                                "source_ref": b.source_ref,
                                "preview": redacted_preview(&buf, class, 2000),
                            }));
                            idx += 1;
                            buf.clear();
                            if chunks.len() >= 20 {
                                break;
                            }
                        }
                        if !buf.is_empty() {
                            buf.push('\n');
                        }
                        buf.push_str(&b.text);
                    }
                    if !buf.is_empty() && chunks.len() < 20 {
                        chunks.push(serde_json::json!({"chunk_index": idx, "chars": buf.len(), "preview": redacted_preview(&buf, class, 2000)}));
                    }
                    Ok(serde_json::json!({
                        "path": p,
                        "chunks": chunks.len(),
                        "classification": class.to_string(),
                        "chunk_previews": chunks,
                    }))
                }
                _ => Ok(serde_json::json!({"path": p, "status": "UNPARSEABLE"})),
            }
        }
        "document.create_artifact" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemWrite,
                &ToolCapability::WorkspaceWrite,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'content'".to_string()))?;
            let target = resolve(ctx, p)?;
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError::ToolExecutionFailed(format!("mkdir failed: {}", e)))?;
            }
            if content.len() > 10 * 1024 * 1024 {
                return Err(ToolError::ToolExecutionFailed(
                    "Artifact exceeds 10MB tool cap".to_string(),
                ));
            }
            fs::write(&target, content)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("write failed: {}", e)))?;
            Ok(serde_json::json!({
                "path": p,
                "bytes_written": content.len(),
                "content_hash": content_hash_hex(content.as_bytes()),
                "verification_status": "UNVERIFIED",
                "note": "Artifact created UNVERIFIED; VerificationEngine must pass before claiming completion",
            }))
        }
        "document.export_artifact" => {
            require_cap(
                ctx,
                &ToolCapability::FilesystemExport,
                &ToolCapability::WorkspaceExport,
            )?;
            let p = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'path'".to_string()))?;
            let dest = args
                .get("dest")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArguments("Missing 'dest'".to_string()))?;
            // Fail closed: only intra-workspace copies allowed from tool context.
            if dest.contains("..")
                || dest.starts_with('/')
                || dest.starts_with('\\')
                || dest.contains(':')
            {
                return Err(ToolError::FilesystemDenied(
                    "External export destinations are blocked in tool context; use the authorized server export API".to_string(),
                ));
            }
            let src = resolve(ctx, p)?;
            let dst = resolve(ctx, dest)?;
            let bytes = fs::read(&src)
                .map_err(|e| ToolError::ToolExecutionFailed(format!("read failed: {}", e)))?;
            let class = classify_text(&String::from_utf8_lossy(&bytes), p);
            if class >= PrivacyClassification::Restricted {
                return Err(ToolError::FilesystemDenied(format!(
                    "Artifact classified {} cannot be exported without authorization",
                    class
                )));
            }
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError::ToolExecutionFailed(format!("mkdir failed: {}", e)))?;
            }
            fs::write(&dst, &bytes).map_err(|e| {
                ToolError::ToolExecutionFailed(format!("export copy failed: {}", e))
            })?;
            Ok(
                serde_json::json!({"path": p, "dest": dest, "bytes": bytes.len(), "status": "COPIED_WITHIN_WORKSPACE"}),
            )
        }
        _ => Err(ToolError::ToolNotFound(tool_id.to_string())),
    }
}
