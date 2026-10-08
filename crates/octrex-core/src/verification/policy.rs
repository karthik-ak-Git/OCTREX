use crate::verification::types::{
    CheckSeverity, CheckStatus, VerificationCheck, VerificationCheckType, VerificationStatus,
};

/// Centralized verification policy.
///
/// Rules:
/// - Any Blocking/Critical failure -> FAIL/BLOCK.
/// - Required artifact missing -> FAIL.
/// - Required build/test failed -> FAIL.
/// - Optional warning -> PASS_WITH_WARNINGS.
/// - Unknown on a security-sensitive check -> BLOCK (fail-closed).
/// - Model claim without evidence -> NOT VERIFIED (FAIL when completion requires evidence).
pub struct VerificationPolicy;

impl VerificationPolicy {
    /// Security-sensitive check types subject to fail-closed Unknown handling.
    pub fn is_security_sensitive(check_type: &VerificationCheckType) -> bool {
        matches!(
            check_type,
            VerificationCheckType::FileExists
                | VerificationCheckType::FileContent
                | VerificationCheckType::FileCreated
                | VerificationCheckType::FileModified
                | VerificationCheckType::ArtifactExists
                | VerificationCheckType::ArtifactValid
                | VerificationCheckType::ConstraintSatisfied
                | VerificationCheckType::NoBlockingError
                | VerificationCheckType::ToolSucceeded
        )
    }

    /// Aggregate per-check outcomes into a run status.
    pub fn aggregate(checks: &[VerificationCheck]) -> VerificationStatus {
        if checks.is_empty() {
            return VerificationStatus::Unknown;
        }

        let mut has_warning = false;

        for check in checks {
            match check.status {
                CheckStatus::Blocked => return VerificationStatus::Blocked,
                CheckStatus::Failed => {
                    if check.severity == CheckSeverity::Blocking
                        || check.severity == CheckSeverity::Critical
                    {
                        return VerificationStatus::Fail;
                    }
                    // Non-critical failures still fail the run unless explicitly
                    // downgraded by the caller to a warning.
                    return VerificationStatus::Fail;
                }
                CheckStatus::Unknown => {
                    if Self::is_security_sensitive(&check.check_type)
                        || check.severity == CheckSeverity::Blocking
                        || check.severity == CheckSeverity::Critical
                    {
                        return VerificationStatus::Blocked;
                    }
                    return VerificationStatus::Fail;
                }
                CheckStatus::Warning => {
                    has_warning = true;
                }
                CheckStatus::Passed => {}
                CheckStatus::Skipped => {}
            }
        }

        if has_warning {
            VerificationStatus::PassWithWarnings
        } else {
            VerificationStatus::Pass
        }
    }

    /// Confidence reflects authoritative evidence only.
    /// Model claims never raise confidence.
    pub fn confidence(checks: &[VerificationCheck], has_model_claim_only: bool) -> f64 {
        if checks.is_empty() {
            return 0.0;
        }
        if has_model_claim_only {
            return 0.0;
        }
        let total = checks.len() as f64;
        let passed = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Passed))
            .count() as f64;
        let warnings = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Warning))
            .count() as f64;
        ((passed + warnings * 0.5) / total).clamp(0.0, 1.0)
    }
}
