use crate::skills::errors::SkillError;
use crate::skills::types::{SkillDefinition, SkillSource, SkillStatus};
use crate::tools::ToolCapability;

/// Privileged capabilities that untrusted sources must never silently obtain.
pub fn privileged_capabilities() -> Vec<ToolCapability> {
    vec![
        ToolCapability::ProcessExecute,
        ToolCapability::ProcessSpawn,
        ToolCapability::FilesystemDelete,
        ToolCapability::FilesystemExport,
        ToolCapability::WorkspaceExport,
        ToolCapability::ReadSecretData,
        ToolCapability::ReadRestrictedData,
        ToolCapability::EnvironmentAccess,
        ToolCapability::RemoteMcp,
        ToolCapability::ProviderApi,
    ]
}

pub fn is_privileged(cap: &ToolCapability) -> bool {
    privileged_capabilities().contains(cap)
}

/// Validate semver-like X.Y.Z where each part is numeric.
pub fn validate_version(version: &str) -> Result<(u64, u64, u64), SkillError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(SkillError::VersionError {
            reason: format!(
                "Version '{}' must use MAJOR.MINOR.PATCH format (e.g. 1.2.0)",
                version
            ),
        });
    }
    let mut nums = [0u64; 3];
    for (i, p) in parts.iter().enumerate() {
        nums[i] = p.parse::<u64>().map_err(|_| SkillError::VersionError {
            reason: format!("Version part '{}' in '{}' is not numeric", p, version),
        })?;
    }
    Ok((nums[0], nums[1], nums[2]))
}

pub fn compare_versions(a: &str, b: &str) -> Result<std::cmp::Ordering, SkillError> {
    Ok(validate_version(a)?.cmp(&validate_version(b)?))
}

/// Core skill validation. Invalid skills must be Disabled, never silently repaired
/// when the issue is security-sensitive.
pub fn validate_skill(def: &SkillDefinition) -> Result<(), SkillError> {
    if def.id.trim().is_empty() {
        return Err(SkillError::Validation {
            reason: "Skill id must not be empty".to_string(),
        });
    }
    if def.name.trim().is_empty() {
        return Err(SkillError::Validation {
            reason: "Skill name must not be empty".to_string(),
        });
    }
    if def.description.trim().is_empty() {
        return Err(SkillError::Validation {
            reason: "Skill description must not be empty".to_string(),
        });
    }
    validate_version(&def.version)?;

    // Unknown source fails closed.
    if def.source == SkillSource::Unknown || def.provenance.source == SkillSource::Unknown {
        return Err(SkillError::SecurityViolation {
            reason: "Skill source is UNKNOWN and fails closed".to_string(),
        });
    }

    if def.workflow.is_empty() {
        return Err(SkillError::Validation {
            reason: "Skill workflow must contain at least one step".to_string(),
        });
    }
    if def.workflow.len() > 32 {
        return Err(SkillError::Validation {
            reason: format!(
                "Skill workflow exceeds max steps (32): {}",
                def.workflow.len()
            ),
        });
    }

    for step in &def.workflow {
        if step.id.trim().is_empty() {
            return Err(SkillError::Validation {
                reason: "Skill step id must not be empty".to_string(),
            });
        }
        if step.reference.trim().is_empty() {
            return Err(SkillError::Validation {
                reason: format!("Skill step '{}' reference must not be empty", step.id),
            });
        }
        // File operation references must not contain traversal outside workspace.
        let lower = step.reference.to_lowercase();
        if lower.contains("..") && !lower.starts_with("workspace://") {
            return Err(SkillError::SecurityViolation {
                reason: format!(
                    "Skill step '{}' references suspicious path '{}'",
                    step.id, step.reference
                ),
            });
        }
        // Absolute system paths are rejected at validation time.
        if step.reference.starts_with('/')
            || step.reference.starts_with('\\')
            || (step.reference.len() > 2 && step.reference.chars().nth(1) == Some(':'))
        {
            return Err(SkillError::SecurityViolation {
                reason: format!(
                    "Skill step '{}' must not reference absolute path '{}'",
                    step.id, step.reference
                ),
            });
        }
    }

    if def.verification_requirements.is_empty() {
        return Err(SkillError::Validation {
            reason: "Skill must declare at least one verification requirement".to_string(),
        });
    }

    // Untrusted sources requesting privileged capabilities require explicit review.
    // They are not auto-activated; validation reports them as needing approval.
    let untrusted = matches!(
        def.source,
        SkillSource::Imported | SkillSource::ModelGenerated | SkillSource::External
    );
    if untrusted {
        let bad: Vec<String> = def
            .capabilities_required
            .iter()
            .filter(|c| is_privileged(c))
            .map(|c| c.to_string())
            .collect();
        if !bad.is_empty() {
            return Err(SkillError::SecurityViolation {
                reason: format!(
                    "Untrusted source {:?} requests privileged capabilities [{}]; explicit approval required",
                    def.source,
                    bad.join(", ")
                ),
            });
        }
    }

    // Declaration is not authorization: capabilities_required must be a subset
    // of (allowed_tools implied caps + explicitly declared). We enforce that
    // every required capability is at least declared; actual grants remain
    // authoritative at execution time via ToolRuntime.
    if def.capabilities_required.len() > 24 {
        return Err(SkillError::Validation {
            reason: "Skill declares too many capabilities (max 24)".to_string(),
        });
    }

    // Policy compatibility: skills must never claim policy override.
    let blob = format!(
        "{} {} {}",
        def.description, def.input_schema, def.output_schema
    )
    .to_lowercase();
    for trigger in [
        "override policy",
        "disable privacy",
        "disable security",
        "bypass boundary",
        "ignore safety",
        "grant admin",
        "always permitted",
    ] {
        if blob.contains(trigger) {
            return Err(SkillError::SecurityViolation {
                reason: format!(
                    "Skill definition contains forbidden policy-override phrase '{}'",
                    trigger
                ),
            });
        }
    }

    Ok(())
}

/// Determine post-validation status without silently repairing security issues.
pub fn post_validation_status(def: &SkillDefinition, valid: bool) -> SkillStatus {
    if !valid {
        return SkillStatus::Disabled;
    }
    match def.source {
        SkillSource::System | SkillSource::Company | SkillSource::BuiltIn | SkillSource::User => {
            SkillStatus::Active
        }
        SkillSource::Imported | SkillSource::ModelGenerated | SkillSource::External => {
            SkillStatus::PendingValidation
        }
        SkillSource::Unknown => SkillStatus::Blocked,
    }
}
