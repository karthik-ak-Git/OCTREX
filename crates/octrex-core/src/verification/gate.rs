use crate::verification::types::{CompletionDecision, VerificationStatus};

/// Centralized completion gate. The orchestrator must consult this before
/// marking a task Completed.
///
/// - Pass -> AllowCompletion
/// - PassWithWarnings -> AllowCompletion (warnings surfaced to user)
/// - Fail with repairable state and remaining attempts -> RequireRepair
/// - Fail with exhausted budget or user-gated state -> RequireUser
/// - Blocked -> Blocked
/// - Unknown -> Blocked (fail-closed; never auto-pass)
pub struct CompletionGate;

pub struct GateInput {
    pub status: VerificationStatus,
    pub repair_attempt: u32,
    pub max_repair_attempts: u32,
    pub has_blocking_failure: bool,
    pub requires_user_input: bool,
}

impl CompletionGate {
    pub fn evaluate(input: &GateInput) -> CompletionDecision {
        match input.status {
            VerificationStatus::Pass | VerificationStatus::PassWithWarnings => {
                CompletionDecision::AllowCompletion
            }
            VerificationStatus::Blocked => CompletionDecision::Blocked,
            VerificationStatus::Unknown => CompletionDecision::Blocked,
            VerificationStatus::Fail => {
                if input.has_blocking_failure {
                    return CompletionDecision::Blocked;
                }
                if input.requires_user_input {
                    return CompletionDecision::RequireUser;
                }
                if input.repair_attempt < input.max_repair_attempts {
                    CompletionDecision::RequireRepair
                } else {
                    CompletionDecision::RequireUser
                }
            }
        }
    }

    pub fn max_repair_default() -> u32 {
        3
    }
}
