use crate::skills::errors::SkillError;
use crate::skills::types::{SkillDefinition, SkillSource, SkillStatus};
use crate::skills::validator;

/// Explicit trust hierarchy. Memory/skills never outrank system policy.
pub fn source_trust_rank(source: SkillSource) -> u8 {
    source.trust_rank()
}

/// Security eligibility comes before any ranking. Returns Ok(()) when the
/// skill is eligible for matching under current conditions.
pub fn check_eligibility(def: &SkillDefinition) -> Result<(), SkillError> {
    if def.status != SkillStatus::Active {
        return Err(SkillError::NotEligible {
            reason: format!(
                "Skill '{}' status is {} (requires ACTIVE)",
                def.id, def.status
            ),
        });
    }
    if def.source == SkillSource::Unknown {
        return Err(SkillError::NotEligible {
            reason: "Skill source UNKNOWN fails closed".to_string(),
        });
    }
    // Untrusted sources must have passed explicit approval before activation.
    // Registry enforces this by keeping them PendingValidation; double-check here.
    if matches!(
        def.source,
        SkillSource::Imported | SkillSource::ModelGenerated | SkillSource::External
    ) && def.provenance.approval_status != "APPROVED"
    {
        return Err(SkillError::NotEligible {
            reason: format!(
                "Untrusted skill '{}' requires explicit APPROVED provenance",
                def.id
            ),
        });
    }
    validator::validate_skill(def).map_err(|e| SkillError::NotEligible {
        reason: e.to_string(),
    })?;
    Ok(())
}

/// Policy compatibility: skill classification must be enforceable and the
/// definition must not attempt to escalate above its source trust.
pub fn check_policy_compatibility(def: &SkillDefinition) -> Result<(), SkillError> {
    // SECRET skills from untrusted sources are never compatible.
    if def.classification == crate::privacy::PrivacyClassification::Secret
        && !def.source.is_auto_trusted()
    {
        return Err(SkillError::SecurityViolation {
            reason: "Untrusted source cannot publish SECRET classification skills".to_string(),
        });
    }
    Ok(())
}
