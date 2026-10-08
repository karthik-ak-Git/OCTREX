use crate::skills::registry::SkillRegistry;
use crate::tools::{ToolId, ToolRegistry};
use crate::workflows::errors::WorkflowError;
use crate::workflows::planner::WorkflowPlanner;
use crate::workflows::registry::WorkflowRegistry;
use crate::workflows::types::{
    now_millis, WorkflowDefinition, WorkflowRun, WorkflowRunStatus, WorkflowStepKind,
};

/// Thin delegation executor. It does NOT implement a second orchestration
/// loop: it validates the workflow (DAG, refs, caps), pins the exact version,
/// creates a Task + run record (Phase 11 TaskRegistry semantics), and returns
/// an ordered, policy-checked step list for the Phase 11 Orchestrator to drive
/// through ContextEngine -> ModelRouter -> ToolRuntime -> Verification.
///
/// Callers (ApplicationState / API) then drive each step through the
/// authoritative boundaries; this struct never executes tools directly.
pub struct WorkflowExecutor;

impl WorkflowExecutor {
    pub fn authorize(
        workflow_registry: &WorkflowRegistry,
        skill_registry: &SkillRegistry,
        tool_registry: &ToolRegistry,
        workflow_id: &str,
        pinned_version: Option<&str>,
    ) -> Result<WorkflowDefinition, WorkflowError> {
        let def = if let Some(v) = pinned_version {
            workflow_registry
                .get_version(workflow_id, v)
                .ok_or_else(|| WorkflowError::NotFound {
                    workflow_id: format!("{}@{}", workflow_id, v),
                })?
        } else {
            workflow_registry
                .get(workflow_id)
                .ok_or_else(|| WorkflowError::NotFound {
                    workflow_id: workflow_id.to_string(),
                })?
        };

        // Selection != authorization: validate again against current policy.
        WorkflowPlanner::authorize(&def)?;

        // Validate references against live registries. Unknown refs fail closed.
        for step in &def.steps {
            match step.kind {
                WorkflowStepKind::Tool => {
                    let tid = ToolId::new(&step.reference);
                    if tool_registry.get(&tid).is_none() {
                        return Err(WorkflowError::CapabilityDenied {
                            reason: format!(
                                "Workflow step '{}' references unregistered tool '{}'",
                                step.id, step.reference
                            ),
                        });
                    }
                }
                WorkflowStepKind::Skill => {
                    // Pinned skill versions are resolved at run creation; here we
                    // require the skill to exist and be eligible.
                    let skill = skill_registry.get(&step.reference).ok_or_else(|| {
                        WorkflowError::CapabilityDenied {
                            reason: format!(
                                "Workflow step '{}' references unknown skill '{}'",
                                step.id, step.reference
                            ),
                        }
                    })?;
                    crate::skills::policy::check_eligibility(&skill).map_err(|e| {
                        WorkflowError::CapabilityDenied {
                            reason: e.to_string(),
                        }
                    })?;
                }
                WorkflowStepKind::FileOperation => {
                    // Cross-workspace access is denied at run time; here we only
                    // reject absolute paths and traversal.
                    let r = step.reference.clone();
                    if r.starts_with('/') || r.starts_with('\\') || r.contains("..") {
                        return Err(WorkflowError::SecurityViolation {
                            reason: format!(
                                "Workflow step '{}' has unsafe file reference",
                                step.id
                            ),
                        });
                    }
                }
                WorkflowStepKind::ModelOperation
                | WorkflowStepKind::Verification
                | WorkflowStepKind::UserInput => {}
            }
        }

        Ok(def)
    }

    /// Create a run record pinning the exact workflow version (no silent
    /// mutation of active tasks when the definition later changes).
    pub fn create_run(
        workflow_registry: &WorkflowRegistry,
        def: &WorkflowDefinition,
        task_id: Option<String>,
        session_id: Option<String>,
        workspace_id: Option<String>,
        inputs: serde_json::Value,
    ) -> Result<WorkflowRun, WorkflowError> {
        let now = now_millis();
        let run = WorkflowRun {
            id: format!("wfrun-{}", uuid::Uuid::new_v4().simple()),
            workflow_id: def.id.clone(),
            workflow_version: def.version.clone(),
            task_id,
            session_id,
            workspace_id,
            status: WorkflowRunStatus::Running,
            inputs,
            outputs: None,
            created_at: now,
            updated_at: now,
        };
        workflow_registry.record_run(&run)?;
        Ok(run)
    }

    /// Ordered steps for the Phase 11 Orchestrator (topological order).
    pub fn ordered_steps(def: &WorkflowDefinition) -> Vec<&crate::workflows::types::WorkflowStep> {
        // Steps are stored in author order; dependencies form a DAG. Kahn's
        // algorithm returns a deterministic execution order.
        use std::collections::{HashMap, VecDeque};
        let mut indegree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
        for s in &def.steps {
            indegree.entry(s.id.as_str()).or_insert(0);
            for dep in &s.dependencies {
                *indegree.entry(s.id.as_str()).or_insert(0) += 1;
                dependents
                    .entry(dep.as_str())
                    .or_default()
                    .push(s.id.as_str());
            }
        }
        let mut queue: VecDeque<&str> = indegree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();
        // Deterministic: sort initial queue.
        let mut sorted: Vec<&str> = queue.drain(..).collect();
        sorted.sort_unstable();
        queue = sorted.into_iter().collect();
        let index: HashMap<&str, &crate::workflows::types::WorkflowStep> =
            def.steps.iter().map(|s| (s.id.as_str(), s)).collect();
        let mut out = Vec::new();
        while let Some(id) = queue.pop_front() {
            if let Some(step) = index.get(id) {
                out.push(*step);
            }
            if let Some(children) = dependents.get(id) {
                let mut kids = children.clone();
                kids.sort_unstable();
                for child in kids {
                    if let Some(d) = indegree.get_mut(child) {
                        *d = d.saturating_sub(1);
                        if *d == 0 {
                            queue.push_back(child);
                        }
                    }
                }
            }
        }
        // If a cycle slipped through (validator should have rejected), fall
        // back to author order rather than dropping steps silently.
        if out.len() != def.steps.len() {
            def.steps.iter().collect()
        } else {
            out
        }
    }
}
