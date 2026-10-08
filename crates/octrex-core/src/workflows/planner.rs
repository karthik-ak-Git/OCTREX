use crate::workflows::errors::WorkflowError;
use crate::workflows::registry::WorkflowRegistry;
use crate::workflows::types::WorkflowDefinition;
use crate::workflows::validator;

/// Selection != authorization. This planner only ranks; the executor
/// re-validates against current policy before running.
pub struct WorkflowPlanner;

impl WorkflowPlanner {
    pub fn select(
        registry: &WorkflowRegistry,
        user_request: &str,
        limit: usize,
    ) -> Vec<(WorkflowDefinition, f64)> {
        let lower = user_request.to_lowercase();
        let tokens: Vec<&str> = lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 2)
            .collect();
        let mut scored = Vec::new();
        for wf in registry.list() {
            if wf.status != crate::workflows::types::WorkflowStatus::Active {
                continue;
            }
            if validator::validate_workflow(&wf).is_err() {
                continue;
            }
            let hay = format!(
                "{} {}",
                wf.name.to_lowercase(),
                wf.description.to_lowercase()
            );
            let mut hits = 0;
            for t in &tokens {
                if hay.contains(t) {
                    hits += 1;
                }
            }
            let score = if tokens.is_empty() {
                0.0
            } else {
                hits as f64 / tokens.len() as f64
            };
            scored.push((wf, score));
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit.max(1));
        scored
    }

    /// Re-validate immediately before execution.
    pub fn authorize(def: &WorkflowDefinition) -> Result<(), WorkflowError> {
        validator::validate_workflow(def).map_err(|e| WorkflowError::NotEligible {
            reason: e.to_string(),
        })?;
        if def.status != crate::workflows::types::WorkflowStatus::Active {
            return Err(WorkflowError::NotEligible {
                reason: format!("Workflow '{}' is not ACTIVE", def.id),
            });
        }
        // Every tool/skill reference is checked by the executor against the
        // live registries; unknown references fail closed there.
        Ok(())
    }
}
