use crate::workflows::errors::WorkflowError;
use crate::workflows::types::WorkflowDefinition;

/// Workflow-level policy: classification floor, untrusted-source limits.
pub struct WorkflowPolicy;

impl WorkflowPolicy {
    pub fn check(def: &WorkflowDefinition) -> Result<(), WorkflowError> {
        // SECRET workflows from untrusted sources are never allowed.
        if def.classification == crate::privacy::PrivacyClassification::Secret
            && !def.source.is_auto_trusted()
        {
            return Err(WorkflowError::SecurityViolation {
                reason: "Untrusted source cannot publish SECRET workflows".to_string(),
            });
        }
        // Workflows must not claim to override higher policy.
        let blob =
            format!("{} {}", def.description, def.policy_requirements.join(" ")).to_lowercase();
        for trigger in [
            "override policy",
            "disable privacy",
            "bypass boundary",
            "always permitted",
        ] {
            if blob.contains(trigger) {
                return Err(WorkflowError::SecurityViolation {
                    reason: format!("Workflow contains forbidden phrase '{}'", trigger),
                });
            }
        }
        Ok(())
    }
}
