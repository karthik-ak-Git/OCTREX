use super::errors::OrchestrationError;
use super::limits::OrchestratorLimits;

#[derive(Debug, Clone)]
pub struct RetryDecision {
    pub should_retry: bool,
    pub backoff_ms: u64,
    pub reason: String,
}

pub struct RetryManager;

impl RetryManager {
    /// Classify error and decide whether step attempt should be retried
    pub fn evaluate_retry(
        err: &OrchestrationError,
        attempt: u32,
        limits: &OrchestratorLimits,
    ) -> RetryDecision {
        if attempt >= limits.max_retries_per_step {
            return RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: format!(
                    "Exceeded max retries per step ({})",
                    limits.max_retries_per_step
                ),
            };
        }

        match err {
            // Non-retryable security & policy violations
            OrchestrationError::SecurityViolation { reason } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: format!("Security violation is non-retryable: {}", reason),
            },
            OrchestrationError::BoundaryViolation { reason } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: format!("Boundary violation is non-retryable: {}", reason),
            },
            OrchestrationError::InvalidPlan { reason } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: format!("Invalid plan structure is non-retryable: {}", reason),
            },
            OrchestrationError::UserConsentRequired { .. } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: "Requires user interaction".to_string(),
            },
            OrchestrationError::CancellationRequested => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: "Task execution cancelled".to_string(),
            },
            OrchestrationError::LimitExceeded { .. } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: "Limit exceeded is non-retryable".to_string(),
            },
            OrchestrationError::NonRetryableError { reason } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: format!("Non-retryable error: {}", reason),
            },

            // Retryable transient failures
            OrchestrationError::StepFailed { reason, .. } => {
                let backoff_ms = (100 * (2u64.pow(attempt))).min(3000);
                RetryDecision {
                    should_retry: true,
                    backoff_ms,
                    reason: format!("Step failed transiently (attempt {}): {}", attempt, reason),
                }
            }
            OrchestrationError::Internal { message } => {
                let backoff_ms = (100 * (2u64.pow(attempt))).min(3000);
                RetryDecision {
                    should_retry: true,
                    backoff_ms,
                    reason: format!("Transient internal error: {}", message),
                }
            }
            OrchestrationError::VerificationFailed { reason, .. } => {
                let backoff_ms = (200 * (2u64.pow(attempt))).min(3000);
                RetryDecision {
                    should_retry: true,
                    backoff_ms,
                    reason: format!("Verification failed, retrying step: {}", reason),
                }
            }
            OrchestrationError::InvalidStateTransition { .. } => RetryDecision {
                should_retry: false,
                backoff_ms: 0,
                reason: "State machine invalid transition is non-retryable".to_string(),
            },
        }
    }
}
