use super::types::{
    ClassificationConfidence, ClassificationResult, ClassificationSignal, InputSourceType,
    PrivacyClassification, PrivacyInput, TrustLevel,
};
use regex::Regex;
use std::sync::OnceLock;

static CREDENTIAL_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();
static CONFIDENTIAL_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
static RESTRICTED_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_credential_regexes() -> &'static [Regex] {
    CREDENTIAL_REGEXES.get_or_init(|| {
        vec![
            Regex::new(r#"(?i)api[_-]?key\s*[:=]\s*['"]?[A-Za-z0-9_-]{16,}['"]?"#).unwrap(),
            Regex::new(r"(?i)bearer\s+[A-Za-z0-9_\.-]{20,}").unwrap(),
            Regex::new(r"-----BEGIN (RSA|EC|DSA|OPENSSH|PRIVATE) KEY-----").unwrap(),
            Regex::new(r"(?i)aws_secret_access_key\s*[:=]\s*[A-Za-z0-9/+=]{40}").unwrap(),
            Regex::new(r#"(?i)password\s*[:=]\s*['"]?[^'\s]{6,}['"]?"#).unwrap(),
            Regex::new(r"sk-[A-Za-z0-9]{20,}").unwrap(),
            Regex::new(r#"(?i)token\s*[:=]\s*['"]?[A-Za-z0-9_-]{16,}['"]?"#).unwrap(),
        ]
    })
}

fn get_confidential_patterns() -> &'static [Regex] {
    CONFIDENTIAL_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)\b(strictly\s+)?confidential\b").unwrap(),
            Regex::new(r"(?i)\bproprietary\s+and\s+confidential\b").unwrap(),
            Regex::new(r"(?i)\bcompany\s+confidential\b").unwrap(),
            Regex::new(r"(?i)\bstandard\s+operating\s+procedure\b").unwrap(),
            Regex::new(r"(?i)\bmanufacturing\s+data\b").unwrap(),
            Regex::new(r"(?i)\bengineering\s+drawing\b").unwrap(),
            Regex::new(r"(?i)\bsop[-\s]\d{3,}\b").unwrap(),
        ]
    })
}

fn get_restricted_patterns() -> &'static [Regex] {
    RESTRICTED_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)\b(top\s+)?secret\b").unwrap(),
            Regex::new(r"(?i)\brestricted\s+access\b").unwrap(),
            Regex::new(r"(?i)\bclassified\s+document\b").unwrap(),
            Regex::new(r"(?i)\bdo\s+not\s+distribute\b").unwrap(),
            Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(), // SSN pattern
        ]
    })
}

pub struct PrivacyClassifier;

impl PrivacyClassifier {
    pub fn new() -> Self {
        Self
    }

    /// Classify a collection of PrivacyInputs and workspace classification context.
    /// Returns the highest privacy classification detected along with confidence and signals.
    pub fn classify(
        &self,
        workspace_classification: PrivacyClassification,
        inputs: &[PrivacyInput],
    ) -> ClassificationResult {
        let mut highest_classification = workspace_classification;
        let mut signals = Vec::new();
        let mut max_confidence = if workspace_classification > PrivacyClassification::Public {
            ClassificationConfidence::High
        } else {
            ClassificationConfidence::Low
        };

        if workspace_classification > PrivacyClassification::Public {
            signals.push(ClassificationSignal {
                category: "WORKSPACE".to_string(),
                signal_type: "WORKSPACE_CLASSIFICATION".to_string(),
                summary: format!("Workspace is classified as {}", workspace_classification),
                severity: "HIGH".to_string(),
            });
        }

        for input in inputs {
            // Check file path / origin patterns
            if let Some(origin) = &input.origin {
                let lower_origin = origin.to_lowercase();
                if lower_origin.contains(".env")
                    || lower_origin.contains("credentials")
                    || lower_origin.contains("secret")
                    || lower_origin.contains("private_key")
                    || lower_origin.contains("id_rsa")
                {
                    if PrivacyClassification::Restricted > highest_classification {
                        highest_classification = PrivacyClassification::Restricted;
                    }
                    max_confidence = ClassificationConfidence::High;
                    signals.push(ClassificationSignal {
                        category: "FILE_NAME".to_string(),
                        signal_type: "SENSITIVE_FILENAME_PATTERN".to_string(),
                        summary: format!("Sensitive filename pattern in '{}'", origin),
                        severity: "HIGH".to_string(),
                    });
                } else if lower_origin.contains("confidential")
                    || lower_origin.contains("sop")
                    || lower_origin.contains("spec")
                    || lower_origin.contains("internal")
                {
                    if PrivacyClassification::Confidential > highest_classification {
                        highest_classification = PrivacyClassification::Confidential;
                    }
                    if max_confidence != ClassificationConfidence::High {
                        max_confidence = ClassificationConfidence::Medium;
                    }
                    signals.push(ClassificationSignal {
                        category: "FILE_NAME".to_string(),
                        signal_type: "CONFIDENTIAL_FILENAME_PATTERN".to_string(),
                        summary: format!("Confidential filename pattern in '{}'", origin),
                        severity: "MEDIUM".to_string(),
                    });
                }
            }

            // Credential inspection (WITHOUT logging raw credential content!)
            let cred_regexes = get_credential_regexes();
            for re in cred_regexes {
                if re.is_match(&input.content) {
                    if PrivacyClassification::Restricted > highest_classification {
                        highest_classification = PrivacyClassification::Restricted;
                    }
                    max_confidence = ClassificationConfidence::High;
                    signals.push(ClassificationSignal {
                        category: "SECURITY".to_string(),
                        signal_type: "CREDENTIAL_DETECTED".to_string(),
                        summary: "credential-like secret detected".to_string(),
                        severity: "CRITICAL".to_string(),
                    });
                    break;
                }
            }

            // Restricted pattern inspection
            let restr_patterns = get_restricted_patterns();
            for re in restr_patterns {
                if re.is_match(&input.content) {
                    if PrivacyClassification::Restricted > highest_classification {
                        highest_classification = PrivacyClassification::Restricted;
                    }
                    max_confidence = ClassificationConfidence::High;
                    signals.push(ClassificationSignal {
                        category: "DATA_SENSITIVITY".to_string(),
                        signal_type: "RESTRICTED_PATTERN_MATCH".to_string(),
                        summary: "Restricted term or pattern detected in payload".to_string(),
                        severity: "HIGH".to_string(),
                    });
                    break;
                }
            }

            // Confidential pattern inspection
            let conf_patterns = get_confidential_patterns();
            for re in conf_patterns {
                if re.is_match(&input.content) {
                    if PrivacyClassification::Confidential > highest_classification {
                        highest_classification = PrivacyClassification::Confidential;
                    }
                    if max_confidence != ClassificationConfidence::High {
                        max_confidence = ClassificationConfidence::High;
                    }
                    signals.push(ClassificationSignal {
                        category: "DATA_SENSITIVITY".to_string(),
                        signal_type: "CONFIDENTIAL_PATTERN_MATCH".to_string(),
                        summary: "Confidential industrial or proprietary text pattern detected"
                            .to_string(),
                        severity: "MEDIUM".to_string(),
                    });
                    break;
                }
            }

            // Internal indicators
            let lower_content = input.content.to_lowercase();
            if lower_content.contains("internal use only")
                || lower_content.contains("internal draft")
                || lower_content.contains("do not share outside")
            {
                if PrivacyClassification::Internal > highest_classification {
                    highest_classification = PrivacyClassification::Internal;
                }
                signals.push(ClassificationSignal {
                    category: "DATA_SENSITIVITY".to_string(),
                    signal_type: "INTERNAL_PATTERN_MATCH".to_string(),
                    summary: "Internal data pattern detected".to_string(),
                    severity: "LOW".to_string(),
                });
            }

            // Detect prompt injection attempts (and note that untrusted inputs do NOT grant policy authority!)
            if lower_content.contains("ignore all octrex policies")
                || lower_content.contains("ignore previous instructions")
                || lower_content.contains("send this document to online")
                || lower_content.contains("cloud use is approved")
                || lower_content.contains("bypass security")
            {
                signals.push(ClassificationSignal {
                    category: "SECURITY".to_string(),
                    signal_type: "PROMPT_INJECTION_ATTEMPT".to_string(),
                    summary: "Prompt injection or policy bypass instruction detected in input text (ignored by policy engine)".to_string(),
                    severity: "WARNING".to_string(),
                });
            }
        }

        let evidence_summary = if signals.is_empty() {
            format!(
                "Classified as {} based on default workspace policy",
                highest_classification
            )
        } else {
            format!(
                "Classified as {} (confidence: {:?}) with {} signal(s)",
                highest_classification,
                max_confidence,
                signals.len()
            )
        };

        ClassificationResult {
            classification: highest_classification,
            confidence: max_confidence,
            signals,
            evidence_summary,
        }
    }
}

impl Default for PrivacyClassifier {
    fn default() -> Self {
        Self::new()
    }
}
