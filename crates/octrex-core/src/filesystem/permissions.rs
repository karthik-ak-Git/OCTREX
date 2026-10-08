use crate::filesystem::policy::PolicyEvaluator;
use crate::filesystem::types::{
    FilesystemDecision, FilesystemDisposition, FilesystemOperation, WorkspaceSecurityPolicy,
};
use crate::ids::WorkspaceId;
use crate::privacy::{PolicySource, PrivacyClassification};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct FilesystemPermissionEngine;

impl FilesystemPermissionEngine {
    pub fn evaluate(
        workspace_id: Option<WorkspaceId>,
        rel_path: &str,
        operation: FilesystemOperation,
        policy: &WorkspaceSecurityPolicy,
        classification: PrivacyClassification,
    ) -> FilesystemDecision {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let risk_level = operation.default_risk();
        let mut warnings = Vec::new();
        let mut matched_policy = None;

        // Check protected path rules
        if let Some((rule, _action)) = PolicyEvaluator::evaluate_protected_path(rel_path, policy) {
            matched_policy = Some(format!("protected_path:{}", rule.pattern));
            warnings.push(format!("Matches protected path pattern '{}'", rule.pattern));
        }

        // Evaluate base policy disposition
        let raw_disposition =
            match PolicyEvaluator::evaluate_policy_disposition(rel_path, operation, policy) {
                Ok(disp) => disp,
                Err(e) => {
                    warnings.push(e.to_string());
                    FilesystemDisposition::Block
                }
            };

        // INVARIANT 2: Unknown disposition MUST result in Block at enforcement time (fail closed).
        let disposition = raw_disposition.sanitize();

        let requires_confirmation =
            matches!(disposition, FilesystemDisposition::RequireConfirmation);

        let reason = match disposition {
            FilesystemDisposition::Allow => {
                format!(
                    "Operation '{}' authorized under workspace security policy",
                    operation
                )
            }
            FilesystemDisposition::Block => {
                if policy.read_only && operation.is_write_like() {
                    "Workspace is set to read-only mode".to_string()
                } else if matched_policy.is_some() {
                    format!(
                        "Access blocked by protected path rule ({})",
                        matched_policy.as_deref().unwrap_or("")
                    )
                } else {
                    format!(
                        "Operation '{}' blocked by workspace security policy",
                        operation
                    )
                }
            }
            FilesystemDisposition::RequireConfirmation => {
                format!(
                    "High-risk operation '{}' requires explicit confirmation",
                    operation
                )
            }
            FilesystemDisposition::Unknown => {
                "Unknown authorization disposition failed closed to Block".to_string()
            }
        };

        FilesystemDecision {
            decision_id: Uuid::new_v4().to_string(),
            timestamp: now,
            decision: disposition,
            operation,
            workspace_id,
            requested_path: rel_path.to_string(),
            resolved_path: None,
            reason,
            matched_policy,
            policy_source: PolicySource::Security,
            classification,
            risk_level,
            warnings,
            requires_confirmation,
        }
    }
}
