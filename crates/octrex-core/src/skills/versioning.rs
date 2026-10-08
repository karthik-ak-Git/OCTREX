use crate::skills::errors::SkillError;
use crate::skills::validator::{compare_versions, validate_version};

/// Parse and compare skill versions. Tasks pin exact versions at start.
pub fn parse_version(version: &str) -> Result<(u64, u64, u64), SkillError> {
    validate_version(version)
}

pub fn is_newer(a: &str, b: &str) -> Result<bool, SkillError> {
    Ok(compare_versions(a, b)? == std::cmp::Ordering::Greater)
}

pub fn versioned_id(id: &str, version: &str) -> String {
    format!("{}@{}", id, version)
}

pub fn parse_versioned_id(versioned: &str) -> Result<(String, String), SkillError> {
    let parts: Vec<&str> = versioned.rsplitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(SkillError::VersionError {
            reason: format!(
                "Versioned id '{}' must look like 'skill-id@1.2.0'",
                versioned
            ),
        });
    }
    validate_version(parts[0])?;
    Ok((parts[1].to_string(), parts[0].to_string()))
}
