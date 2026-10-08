use super::types::{
    ClassificationResult, OutboundPayloadPreview, PrivacyClassification, PrivacyContext,
    PrivacyDecision,
};

pub struct EvidenceManager;

impl EvidenceManager {
    pub fn build_payload_preview(
        context: &PrivacyContext,
        classification: PrivacyClassification,
        class_result: &ClassificationResult,
    ) -> OutboundPayloadPreview {
        let mut data_categories = Vec::new();
        let mut files_included = Vec::new();
        let mut sensitive_fields = Vec::new();
        let mut approximate_bytes = 0;

        for signal in &class_result.signals {
            if !data_categories.contains(&signal.category) {
                data_categories.push(signal.category.clone());
            }
            if !sensitive_fields.contains(&signal.summary) {
                sensitive_fields.push(signal.summary.clone());
            }
        }

        for input in &context.inputs {
            approximate_bytes += input.content.len();
            if let Some(origin) = &input.origin {
                if !files_included.contains(origin) {
                    files_included.push(origin.clone());
                }
            }
        }

        OutboundPayloadPreview {
            request_id: context.request_id.clone(),
            target_provider: context
                .candidate_provider
                .clone()
                .unwrap_or_else(|| "unspecified".to_string()),
            target_model: context
                .candidate_model
                .clone()
                .unwrap_or_else(|| "unspecified".to_string()),
            execution_mode: context.requested_mode,
            classification,
            data_categories,
            files_included,
            approximate_payload_bytes: approximate_bytes,
            sensitive_fields_detected: sensitive_fields,
            policy_decision: super::types::DecisionState::DenyOnline, // placeholder updated by PrivacyGate
        }
    }

    pub fn format_explanation(decision: &PrivacyDecision) -> String {
        format!(
            "Privacy Decision [{}]: {}\nReason: {}\nClassification: {} | Policy Source: {} (v{})",
            decision.decision,
            if decision
                .allowed_execution_modes
                .contains(&decision.requested_execution_mode)
            {
                "ALLOWED"
            } else {
                "RESTRICTED / DENIED"
            },
            decision.reason,
            decision.classification,
            decision.selected_policy_source,
            decision.policy_version
        )
    }

    pub fn redact_string(input: &str) -> String {
        let mut result = input.to_string();
        let credential_patterns = [
            r#"(?i)api[_-]?key\s*[:=]\s*['"]?[A-Za-z0-9_-]{16,}['"]?"#,
            r"(?i)bearer\s+[A-Za-z0-9_\.-]{20,}",
            r"-----BEGIN (RSA|EC|DSA|OPENSSH|PRIVATE) KEY-----",
            r"sk-[A-Za-z0-9_-]{16,}",
            r#"(?i)password\s*[:=]\s*['"]?[^'\s]{6,}['"]?"#,
        ];

        for pat in credential_patterns {
            if let Ok(re) = regex::Regex::new(pat) {
                result = re.replace_all(&result, "[REDACTED_SECRET]").to_string();
            }
        }
        result
    }
}
