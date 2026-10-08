//! Artifact content validation + verification-request builders.
//!
//! Reuses `ExpectedArtifact` / `ExpectedFile` and the existing
//! VerificationEngine. No duplicate verification logic.

use crate::db::models::ArtifactRecord;
use crate::documents::types::ArtifactKind;
use crate::verification::{ExpectedArtifact, ExpectedFile, TaskVerificationRequest};

pub fn artifact_kind_from_str(s: &str) -> Option<ArtifactKind> {
    ArtifactKind::parse_label(s)
}

/// Validate artifact content before writing: extension/content consistency,
/// JSON well-formedness for JSON artifacts, CSV header sanity, size bounds.
/// Returns warnings (never secrets).
pub fn validate_artifact_content(
    kind: &ArtifactKind,
    rel_path: &str,
    content: &str,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    if content.is_empty() {
        return Err("Artifact content must not be empty".to_string());
    }
    if content.len() > 10 * 1024 * 1024 {
        return Err("Artifact exceeds 10MB cap".to_string());
    }
    let lower = rel_path.to_lowercase();
    match kind {
        ArtifactKind::GeneratedJson | ArtifactKind::ExtractedDataset
            if matches!(kind, ArtifactKind::GeneratedJson) =>
        {
            if !lower.ends_with(".json") && !lower.ends_with(".jsonl") {
                warnings.push(format!(
                    "JSON artifact '{}' does not end with .json; keeping declared kind but flagging",
                    rel_path
                ));
            }
            if serde_json::from_str::<serde_json::Value>(content).is_err() {
                return Err("Generated JSON artifact is not valid JSON".to_string());
            }
        }
        ArtifactKind::GeneratedCsv => {
            if !lower.ends_with(".csv") {
                warnings.push(format!("CSV artifact '{}' should end with .csv", rel_path));
            }
            if !content.contains(',') && !content.contains('\n') {
                warnings.push("CSV artifact has no delimiters; may be a single value".to_string());
            }
        }
        ArtifactKind::GeneratedMarkdown | ArtifactKind::GeneratedReport => {
            if !(lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".txt"))
            {
                warnings.push(format!(
                    "Markdown/report artifact '{}' has an unusual extension",
                    rel_path
                ));
            }
        }
        ArtifactKind::GeneratedCode if !rel_path.contains('.') => {
            warnings.push("Code artifact has no file extension".to_string());
        }
        _ => {}
    }
    // Refuse executable-looking artifacts that would execute on open? We never
    // execute; just warn on script extensions so reviewers stay alert.
    for ext in [
        ".exe", ".bat", ".cmd", ".ps1", ".sh", ".dll", ".so", ".dylib",
    ] {
        if lower.ends_with(ext) {
            warnings.push(format!(
                "Artifact '{}' has executable extension {}; it will never be executed by Octrex",
                rel_path, ext
            ));
            break;
        }
    }
    Ok(warnings)
}

/// Build the verification request for an artifact (file must exist,
/// non-empty, registered with matching name/type).
pub fn build_artifact_verification_request(
    artifact: &ArtifactRecord,
    task_id: &str,
    workspace_id: Option<&str>,
    session_id: Option<&str>,
) -> TaskVerificationRequest {
    let mut req = TaskVerificationRequest::for_task(task_id);
    if let Some(w) = workspace_id {
        req.workspace_id = Some(w.to_string());
    }
    if let Some(s) = session_id {
        req.session_id = Some(s.to_string());
    }
    req.expected_artifacts.push(ExpectedArtifact {
        name: artifact.name.clone(),
        artifact_type: Some(artifact.artifact_type.clone()),
        max_size_bytes: Some(10 * 1024 * 1024),
    });
    req.expected_files.push(ExpectedFile {
        rel_path: artifact.path.clone(),
        must_exist: true,
        must_not_be_empty: Some(true),
        expected_extension: None,
        expected_contains: None,
        must_be_modified_after_ms: None,
    });
    req
}
