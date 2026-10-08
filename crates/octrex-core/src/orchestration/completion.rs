use super::errors::OrchestrationError;
use super::types::{StepStatus, TaskPlan};
use std::path::Path;

/// Legacy step/completion verification trait for the Phase 11 orchestrator.
///
/// NOTE (Phase 13): this trait predates the Verification & Reliability Engine
/// (`crate::verification::VerificationEngine`). It remains for step-level
/// gating inside `TaskExecutor`, but it is NOT completion evidence on its own:
/// `TaskExecutor` routes the terminal completion decision through the
/// Verification Engine's `CompletionGate` whenever an engine is attached.
/// `verify_model_result` returning `Ok(true)` only means "a Verify step action
/// may proceed" — it never marks a task complete.
pub trait VerificationService: Send + Sync {
    fn verify_task_step(
        &self,
        plan: &TaskPlan,
        step_index: usize,
    ) -> Result<bool, OrchestrationError>;
    fn verify_artifact(&self, path: &str) -> Result<bool, OrchestrationError>;
    fn verify_model_result(&self, result: &str) -> Result<bool, OrchestrationError>;
    fn verify_completion(&self, plan: &TaskPlan) -> Result<bool, OrchestrationError>;
}

pub struct StandardVerificationService;

impl StandardVerificationService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StandardVerificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationService for StandardVerificationService {
    fn verify_task_step(
        &self,
        plan: &TaskPlan,
        step_index: usize,
    ) -> Result<bool, OrchestrationError> {
        if let Some(step) = plan.steps.get(step_index) {
            if step.status == StepStatus::Failed {
                return Ok(false);
            }

            // If file output required, verify file exists
            if let Some(inputs) = &step.inputs {
                if let Some(filepath) = inputs.get("rel_path").and_then(|v| v.as_str()) {
                    if step.action_type == super::types::StepActionType::WriteFile {
                        let exists = Path::new(filepath).exists();
                        if !exists {
                            return Err(OrchestrationError::VerificationFailed {
                                check_name: "file_exists".to_string(),
                                reason: format!("Required step output file '{}' missing", filepath),
                            });
                        }
                    }
                }
            }

            Ok(true)
        } else {
            Err(OrchestrationError::VerificationFailed {
                check_name: "valid_step_index".to_string(),
                reason: format!("Step index {} out of bounds", step_index),
            })
        }
    }

    /// Fail-closed artifact probe.
    ///
    /// A bare path carries no workspace authorization, so absolute paths and
    /// parent-traversal paths can never be verified here: they resolve outside
    /// any authorized workspace boundary by construction. Callers needing
    /// workspace-scoped artifact verification must use
    /// `crate::verification::VerificationEngine`, which resolves every path
    /// through `FilesystemSecurityService`.
    fn verify_artifact(&self, path: &str) -> Result<bool, OrchestrationError> {
        if path.is_empty() {
            return Ok(false);
        }
        let p = Path::new(path);
        if p.is_absolute() || path.contains("..") {
            return Err(OrchestrationError::VerificationFailed {
                check_name: "artifact_authorized".to_string(),
                reason: format!(
                    "Artifact path '{}' is not workspace-scoped; refusing to verify outside an authorized boundary",
                    path
                ),
            });
        }
        Ok(p.exists())
    }

    /// Step-level model-output gate (NOT completion evidence).
    ///
    /// A non-empty sanitized model response lets a `Verify` step action
    /// proceed. It never establishes task completion: completion requires
    /// `verify_completion` plus the Phase 13 `CompletionGate`.
    fn verify_model_result(&self, result: &str) -> Result<bool, OrchestrationError> {
        // Ensure result is non-empty and does not contain raw error trace
        if result.trim().is_empty() {
            return Ok(false);
        }
        Ok(true)
    }

    fn verify_completion(&self, plan: &TaskPlan) -> Result<bool, OrchestrationError> {
        if plan.steps.is_empty() {
            return Ok(false);
        }

        for step in &plan.steps {
            if step.status != StepStatus::Completed && step.status != StepStatus::Skipped {
                return Err(OrchestrationError::VerificationFailed {
                    check_name: "all_steps_completed".to_string(),
                    reason: format!(
                        "Step '{}' is not completed (status: {})",
                        step.id, step.status
                    ),
                });
            }
        }

        Ok(true)
    }
}
