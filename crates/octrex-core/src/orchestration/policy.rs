use super::errors::OrchestrationError;
use crate::privacy::PrivacyGate;

pub struct OrchestratorPolicyEvaluator;

impl OrchestratorPolicyEvaluator {
    /// Inspect model output proposal for unauthorized policy override attempts.
    /// Returns cleaned text or error if malicious security override detected.
    pub fn validate_model_proposal(
        proposal: &str,
        _privacy_gate: &PrivacyGate,
    ) -> Result<String, OrchestrationError> {
        let lower = proposal.to_lowercase();

        // Check for policy override injection attempts
        let injection_triggers = [
            "disable privacy",
            "disable security",
            "override policy",
            "bypass boundary",
            "ignore safety",
            "grant admin",
            "elevation of privilege",
            "disable network filter",
            "allow all network",
        ];

        for trigger in injection_triggers {
            if lower.contains(trigger) {
                return Err(OrchestrationError::SecurityViolation {
                    reason: format!(
                        "Model output attempted unauthorized security policy alteration: '{}'",
                        trigger
                    ),
                });
            }
        }

        Ok(proposal.to_string())
    }

    /// Ensure tool or file contents are treated as untrusted and sanitized before inclusion
    pub fn sanitize_untrusted_content(content: &str) -> String {
        // Strip potential control characters or raw instruction injection sequences
        content.replace("\0", "").replace("\u{001B}", "") // ANSI escapes
    }
}
