use crate::skills::errors::SkillError;
use crate::skills::types::SkillDefinition;

/// Adversarial checks applied at registration and at execution time.
pub struct SkillSecurity;

impl SkillSecurity {
    /// Reject prompt-injection style instructions that attempt to override
    /// system policy through skill text.
    pub fn scan_for_injection(def: &SkillDefinition) -> Result<(), SkillError> {
        let blob = format!(
            "{} {} {}",
            def.name,
            def.description,
            def.workflow
                .iter()
                .map(|s| format!("{} {}", s.description, s.reference))
                .collect::<Vec<_>>()
                .join(" ")
        )
        .to_lowercase();
        for trigger in [
            "ignore previous instructions",
            "override system",
            "disable privacy gate",
            "bypass filesystem",
            "allow all network",
            "exfiltrate",
            "upload credentials",
            "always permitted",
        ] {
            if blob.contains(trigger) {
                return Err(SkillError::SecurityViolation {
                    reason: format!(
                        "Skill '{}' contains forbidden instruction pattern '{}'",
                        def.id, trigger
                    ),
                });
            }
        }
        Ok(())
    }

    /// A skill with high success metrics is NOT automatically trusted.
    /// Trust derives only from source + validation + approval.
    pub fn trust_from_metrics_is_forbidden() -> bool {
        true
    }
}
