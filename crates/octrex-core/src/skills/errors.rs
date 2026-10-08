use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Skill validation failed: {reason}")]
    Validation { reason: String },

    #[error("Skill not found: {skill_id}")]
    NotFound { skill_id: String },

    #[error("Skill is not eligible for execution: {reason}")]
    NotEligible { reason: String },

    #[error("Skill security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Skill capability denied: {reason}")]
    CapabilityDenied { reason: String },

    #[error("Skill version error: {reason}")]
    VersionError { reason: String },

    #[error("Skill persistence failure: {message}")]
    Persistence { message: String },

    #[error("Internal skill error: {message}")]
    Internal { message: String },
}

impl From<SkillError> for crate::error::OctrexError {
    fn from(e: SkillError) -> Self {
        match &e {
            SkillError::Validation { .. } => crate::error::OctrexError::Validation {
                message: e.to_string(),
                details: None,
            },
            SkillError::NotFound { skill_id } => crate::error::OctrexError::NotFound {
                resource: format!("Skill '{}'", skill_id),
            },
            SkillError::NotEligible { .. }
            | SkillError::SecurityViolation { .. }
            | SkillError::CapabilityDenied { .. } => crate::error::OctrexError::PermissionDenied {
                reason: e.to_string(),
            },
            SkillError::VersionError { .. } => crate::error::OctrexError::Validation {
                message: e.to_string(),
                details: None,
            },
            SkillError::Persistence { .. } | SkillError::Internal { .. } => {
                crate::error::OctrexError::Internal {
                    message: e.to_string(),
                }
            }
        }
    }
}
