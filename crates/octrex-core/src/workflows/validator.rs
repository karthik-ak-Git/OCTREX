use crate::skills::types::SkillSource;
use crate::workflows::errors::WorkflowError;
use crate::workflows::types::{WorkflowDefinition, WorkflowStatus};
use std::collections::{HashMap, HashSet};

pub const MAX_WORKFLOW_STEPS: usize = 32;
pub const MAX_WORKFLOW_DEPTH: usize = 8;

fn validate_version(version: &str) -> Result<(), WorkflowError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.parse::<u64>().is_err()) {
        return Err(WorkflowError::Validation {
            reason: format!("Version '{}' must use MAJOR.MINOR.PATCH format", version),
        });
    }
    Ok(())
}

/// Validate DAG: no unknown deps, no self-deps, no cycles, bounded depth/steps.
pub fn validate_workflow(def: &WorkflowDefinition) -> Result<(), WorkflowError> {
    if def.id.trim().is_empty() {
        return Err(WorkflowError::Validation {
            reason: "Workflow id must not be empty".to_string(),
        });
    }
    if def.name.trim().is_empty() {
        return Err(WorkflowError::Validation {
            reason: "Workflow name must not be empty".to_string(),
        });
    }
    validate_version(&def.version)?;

    if def.source == SkillSource::Unknown {
        return Err(WorkflowError::SecurityViolation {
            reason: "Workflow source UNKNOWN fails closed".to_string(),
        });
    }
    if def.steps.is_empty() {
        return Err(WorkflowError::Validation {
            reason: "Workflow must contain at least one step".to_string(),
        });
    }
    if def.steps.len() > MAX_WORKFLOW_STEPS {
        return Err(WorkflowError::Validation {
            reason: format!(
                "Workflow exceeds max steps ({}): {}",
                MAX_WORKFLOW_STEPS,
                def.steps.len()
            ),
        });
    }

    let mut ids = HashSet::new();
    for step in &def.steps {
        if step.id.trim().is_empty() {
            return Err(WorkflowError::Validation {
                reason: "Workflow step id must not be empty".to_string(),
            });
        }
        if !ids.insert(step.id.clone()) {
            return Err(WorkflowError::Validation {
                reason: format!("Duplicate workflow step id '{}'", step.id),
            });
        }
        if step.reference.trim().is_empty() {
            return Err(WorkflowError::Validation {
                reason: format!("Step '{}' reference must not be empty", step.id),
            });
        }
        let lower = step.reference.to_lowercase();
        if lower.contains("..") && !lower.starts_with("workspace://") {
            return Err(WorkflowError::SecurityViolation {
                reason: format!("Step '{}' has suspicious reference", step.id),
            });
        }
    }

    for step in &def.steps {
        for dep in &step.dependencies {
            if !ids.contains(dep) {
                return Err(WorkflowError::Validation {
                    reason: format!("Step '{}' depends on unknown step '{}'", step.id, dep),
                });
            }
            if dep == &step.id {
                return Err(WorkflowError::Validation {
                    reason: format!("Step '{}' cannot depend on itself", step.id),
                });
            }
        }
    }

    // Cycle detection (DFS over dependency edges).
    let graph: HashMap<&str, Vec<&str>> = def
        .steps
        .iter()
        .map(|s| {
            (
                s.id.as_str(),
                s.dependencies.iter().map(|d| d.as_str()).collect(),
            )
        })
        .collect();
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: HashSet<String> = HashSet::new();
    fn dfs(
        node: &str,
        graph: &HashMap<&str, Vec<&str>>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        stack.insert(node.to_string());
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if !visited.contains(*dep) {
                    if dfs(dep, graph, visited, stack) {
                        return true;
                    }
                } else if stack.contains(*dep) {
                    return true;
                }
            }
        }
        stack.remove(node);
        false
    }
    for step in &def.steps {
        if !visited.contains(&step.id) && dfs(&step.id, &graph, &mut visited, &mut stack) {
            return Err(WorkflowError::Validation {
                reason: format!("Cyclic dependency involving step '{}'", step.id),
            });
        }
    }

    // Depth bound via longest path.
    fn depth(
        node: &str,
        graph: &HashMap<&str, Vec<&str>>,
        memo: &mut HashMap<String, usize>,
    ) -> usize {
        if let Some(cached) = memo.get(node) {
            return *cached;
        }
        let d = match graph.get(node) {
            Some(deps) if !deps.is_empty() => {
                1 + deps
                    .iter()
                    .map(|dep| depth(dep, graph, memo))
                    .max()
                    .unwrap_or(0)
            }
            _ => 1,
        };
        memo.insert(node.to_string(), d);
        d
    }
    let mut memo = HashMap::new();
    let max_depth = def
        .steps
        .iter()
        .map(|s| depth(&s.id, &graph, &mut memo))
        .max()
        .unwrap_or(0);
    if max_depth > MAX_WORKFLOW_DEPTH {
        return Err(WorkflowError::Validation {
            reason: format!(
                "Workflow depth {} exceeds max {}",
                max_depth, MAX_WORKFLOW_DEPTH
            ),
        });
    }

    if def.verification.is_empty() {
        return Err(WorkflowError::Validation {
            reason: "Workflow must declare at least one verification rule".to_string(),
        });
    }

    Ok(())
}

pub fn post_validation_status(def: &WorkflowDefinition, valid: bool) -> WorkflowStatus {
    if !valid {
        return WorkflowStatus::Disabled;
    }
    match def.source {
        SkillSource::System | SkillSource::Company | SkillSource::BuiltIn | SkillSource::User => {
            WorkflowStatus::Active
        }
        SkillSource::Imported | SkillSource::ModelGenerated | SkillSource::External => {
            WorkflowStatus::Draft
        }
        SkillSource::Unknown => WorkflowStatus::Blocked,
    }
}
