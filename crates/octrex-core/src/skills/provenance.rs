use crate::skills::errors::SkillError;
use crate::skills::types::SkillDefinition;
use crate::tools::ToolCapability;

/// Provenance record stored alongside each skill version.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceRecord {
    pub skill_id: String,
    pub version: String,
    pub created_by: String,
    pub source: String,
    pub imported_from: Option<String>,
    pub approval_status: String,
    pub validation_status: String,
    pub tasks_used_in: Vec<String>,
    pub last_updated: u64,
}

impl From<&SkillDefinition> for ProvenanceRecord {
    fn from(def: &SkillDefinition) -> Self {
        Self {
            skill_id: def.id.clone(),
            version: def.version.clone(),
            created_by: def.provenance.created_by.clone(),
            source: def.source.to_string(),
            imported_from: def.provenance.imported_from.clone(),
            approval_status: def.provenance.approval_status.clone(),
            validation_status: def.provenance.validation_status.clone(),
            tasks_used_in: def.provenance.tasks_used_in.clone(),
            last_updated: def.provenance.last_updated,
        }
    }
}

/// Never expose sensitive provenance details unnecessarily: redact creator
/// identity for untrusted/external skills in UI-facing summaries.
pub fn safe_provenance_summary(def: &SkillDefinition) -> serde_json::Value {
    let creator = match def.source {
        crate::skills::types::SkillSource::System
        | crate::skills::types::SkillSource::Company
        | crate::skills::types::SkillSource::BuiltIn
        | crate::skills::types::SkillSource::User => def.provenance.created_by.clone(),
        _ => "[REDACTED: untrusted source]".to_string(),
    };
    serde_json::json!({
        "skill_id": def.id,
        "version": def.version,
        "source": def.source.to_string(),
        "created_by": creator,
        "imported_from": def.provenance.imported_from,
        "approval_status": def.provenance.approval_status,
        "validation_status": def.provenance.validation_status,
        "tasks_used_count": def.provenance.tasks_used_in.len(),
    })
}

/// Capability declarations that would bypass Privacy Gate / Filesystem /
/// Network boundaries are rejected here before execution.
pub fn check_boundary_bypass(
    capabilities: &[ToolCapability],
    declared_tools: &[String],
) -> Result<(), SkillError> {
    // A skill that declares DocumentGeneration + NetworkAccess + FilesystemWrite
    // without any declared tool to back it is suspicious.
    let wants_fs_write = capabilities.contains(&ToolCapability::FilesystemWrite);
    let wants_net = capabilities.contains(&ToolCapability::ExternalHttps)
        || capabilities.contains(&ToolCapability::ExternalHttp)
        || capabilities.contains(&ToolCapability::WebFetch);
    if wants_fs_write && wants_net && declared_tools.is_empty() {
        return Err(SkillError::SecurityViolation {
            reason: "Skill requests filesystem+network capabilities with no backing tool; possible exfiltration pattern"
                .to_string(),
        });
    }
    Ok(())
}
