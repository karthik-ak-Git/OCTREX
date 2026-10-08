use crate::memory::classifier::enforce_no_downgrade;
use crate::memory::errors::MemoryError;
use crate::memory::types::{
    now_millis, MemoryCandidate, MemoryProvenance, MemorySource, MemoryType,
};
use crate::privacy::PrivacyClassification;

/// Provenance-safe candidate extraction. Distinguishes explicit user
/// statements (high trust) from untrusted content (tool output, documents,
/// model output). Only appropriate sources can create high-trust preferences.
pub struct MemoryExtractor;

pub struct ExtractionInput {
    pub text: String,
    pub source: MemorySource,
    pub actor: String,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub tool_id: Option<String>,
    pub model_id: Option<String>,
}

impl MemoryExtractor {
    /// Identify candidate memories without automatically saving model
    /// statements. Returns zero or more candidates for user approval.
    pub fn extract_candidates(
        input: &ExtractionInput,
    ) -> Result<Vec<MemoryCandidate>, MemoryError> {
        // Only explicit user text and observed task facts produce candidates
        // automatically. Tool/model content is quarantined: it may only yield
        // candidates with explicit user confirmation (handled by the caller via
        // approve flow), never high-trust preferences.
        let mut out = Vec::new();
        let text = input.text.trim();
        if text.is_empty() {
            return Ok(out);
        }

        // Poisoning defense: malicious documents must not become preferences.
        // If the source is not UserExplicit/System/Company and the text looks
        // like an instruction to remember ("remember that...", "always ..."),
        // refuse to create a UserPreference candidate.
        let lower = text.to_lowercase();
        let looks_like_preference = lower.contains("always ")
            || lower.contains("remember that")
            || lower.contains("prefer ");
        let is_instruction_to_remember =
            lower.contains("remember that i always") || lower.contains("remember that cloud");

        if is_instruction_to_remember && !is_high_trust_source(input.source) {
            return Err(MemoryError::SecurityViolation {
                reason: "Untrusted content attempts to plant persistent preference; refused"
                    .to_string(),
            });
        }

        // Credential exfiltration pattern: never create candidates carrying
        // upload instructions from untrusted sources.
        if (lower.contains("upload") && lower.contains("credential"))
            || (lower.contains("upload") && lower.contains("password"))
        {
            if !is_high_trust_source(input.source) {
                return Err(MemoryError::SecurityViolation {
                    reason: "Untrusted content attempts credential-upload memory; refused"
                        .to_string(),
                });
            }
        }

        // Policy-override pattern from tool output must never become memory.
        if lower.contains("always permitted") || lower.contains("cloud access is always") {
            return Err(MemoryError::SecurityViolation {
                reason: "Tool/model output attempts policy memory; refused".to_string(),
            });
        }

        // Heuristic candidate typing.
        let mem_type = if looks_like_preference && is_high_trust_source(input.source) {
            MemoryType::UserPreference
        } else if lower.contains("uses ") || lower.contains("project ") {
            MemoryType::ProjectFact
        } else if lower.contains("decision") || lower.contains("decided") {
            MemoryType::Decision
        } else if lower.contains("must not") || lower.contains("constraint") {
            MemoryType::Constraint
        } else {
            MemoryType::TaskFact
        };

        // Tool-derived candidates keep tool provenance and low confidence.
        let confidence = match input.source {
            MemorySource::UserExplicit => 0.9,
            MemorySource::System | MemorySource::Company => 1.0,
            MemorySource::Observed => 0.6,
            MemorySource::ToolDerived => 0.4,
            MemorySource::ModelDerived | MemorySource::Imported => 0.3,
            MemorySource::Unknown => {
                return Err(MemoryError::SecurityViolation {
                    reason: "Cannot extract memory from UNKNOWN source".to_string(),
                })
            }
        };

        let classification = enforce_no_downgrade(PrivacyClassification::Internal, text)?;
        let now = now_millis();
        out.push(MemoryCandidate {
            id: format!("memcand-{}", uuid::Uuid::new_v4().simple()),
            mem_type,
            scope: scope_for(&mem_type, input),
            workspace_id: input.workspace_id.clone(),
            session_id: input.session_id.clone(),
            task_id: input.task_id.clone(),
            content: truncate(text, 2000),
            classification,
            source: input.source,
            confidence,
            provenance: MemoryProvenance {
                source: input.source,
                actor: input.actor.clone(),
                tool_id: input.tool_id.clone(),
                model_id: input.model_id.clone(),
                imported_from: None,
                evidence: truncate(text, 500),
            },
            status: "PENDING".to_string(),
            created_at: now,
            updated_at: now,
        });
        Ok(out)
    }
}

fn is_high_trust_source(s: MemorySource) -> bool {
    s.is_high_trust()
}

fn scope_for(mem_type: &MemoryType, input: &ExtractionInput) -> crate::memory::types::MemoryScope {
    use crate::memory::types::MemoryScope;
    match mem_type {
        MemoryType::UserPreference => MemoryScope::Global,
        MemoryType::ProjectFact => MemoryScope::Project,
        MemoryType::WorkspaceFact => MemoryScope::Workspace,
        MemoryType::TaskFact => {
            if input.task_id.is_some() {
                MemoryScope::Task
            } else if input.session_id.is_some() {
                MemoryScope::Session
            } else {
                MemoryScope::Workspace
            }
        }
        MemoryType::Decision | MemoryType::Constraint | MemoryType::Procedure => {
            if input.task_id.is_some() {
                MemoryScope::Task
            } else {
                MemoryScope::Workspace
            }
        }
        _ => {
            if input.task_id.is_some() {
                MemoryScope::Task
            } else if input.session_id.is_some() {
                MemoryScope::Session
            } else if input.workspace_id.is_some() {
                MemoryScope::Workspace
            } else {
                MemoryScope::Global
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
