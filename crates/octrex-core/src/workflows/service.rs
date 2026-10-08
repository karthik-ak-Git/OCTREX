use crate::db::manager::DatabaseManager;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::skills::registry::SkillRegistry;
use crate::tools::ToolRegistry;
use crate::workflows::errors::WorkflowError;
use crate::workflows::executor::WorkflowExecutor;
use crate::workflows::planner::WorkflowPlanner;
use crate::workflows::registry::WorkflowRegistry;
use crate::workflows::types::{now_millis, WorkflowDefinition, WorkflowRun, WorkflowRunStatus};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkflowService {
    registry: WorkflowRegistry,
    event_bus: Arc<EventBus>,
    db: Arc<DatabaseManager>,
}

impl WorkflowService {
    pub fn new(db: Arc<DatabaseManager>, event_bus: Arc<EventBus>) -> Self {
        Self {
            registry: WorkflowRegistry::new(db.clone()),
            event_bus,
            db,
        }
    }

    pub fn registry(&self) -> &WorkflowRegistry {
        &self.registry
    }

    fn emit(&self, event_type: EventType, payload: serde_json::Value) {
        let _ = self
            .event_bus
            .publish(EventEnvelope::new(event_type, payload));
    }

    fn audit(&self, event_type: &str, workflow_id: &str, success: bool, reason: &str) {
        let id = format!("audit-{}", uuid::Uuid::new_v4().simple());
        let now = now_millis();
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_records (id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason)
                 VALUES (?1, ?2, ?3, NULL, NULL, NULL, 'workflow_service', NULL, NULL, NULL, NULL, NULL, NULL, ?4, ?5, ?6)",
                rusqlite::params![id, now as i64, event_type, workflow_id, if success { 1 } else { 0 }, reason],
            )
            .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
            Ok(())
        });
    }

    pub fn create(&self, def: WorkflowDefinition) -> Result<WorkflowDefinition, WorkflowError> {
        crate::workflows::policy::WorkflowPolicy::check(&def)?;
        let stored = self.registry.register(def)?;
        self.emit(
            EventType::WorkflowCreated,
            serde_json::json!({"workflow_id": stored.id, "version": stored.version}),
        );
        self.audit("WORKFLOW_CREATED", &stored.id, true, "created");
        Ok(stored)
    }

    pub fn validate(&self, id: &str) -> Result<bool, WorkflowError> {
        let def = self
            .registry
            .get(id)
            .ok_or_else(|| WorkflowError::NotFound {
                workflow_id: id.to_string(),
            })?;
        match crate::workflows::validator::validate_workflow(&def) {
            Ok(()) => Ok(true),
            Err(e) => Err(e),
        }
    }

    pub fn select(&self, user_request: &str, limit: usize) -> Vec<(WorkflowDefinition, f64)> {
        WorkflowPlanner::select(&self.registry, user_request, limit)
    }

    /// Authorize + pin version + create run record. The returned ordered steps
    /// are driven by the Phase 11 Orchestrator through authoritative runtimes.
    #[allow(clippy::too_many_arguments)]
    pub fn start_run(
        &self,
        skill_registry: &SkillRegistry,
        tool_registry: &ToolRegistry,
        workflow_id: &str,
        pinned_version: Option<&str>,
        task_id: Option<String>,
        session_id: Option<String>,
        workspace_id: Option<String>,
        inputs: serde_json::Value,
    ) -> Result<(WorkflowRun, serde_json::Value), WorkflowError> {
        let def = WorkflowExecutor::authorize(
            &self.registry,
            skill_registry,
            tool_registry,
            workflow_id,
            pinned_version,
        )?;
        let run = WorkflowExecutor::create_run(
            &self.registry,
            &def,
            task_id,
            session_id,
            workspace_id,
            inputs,
        )?;
        let steps = WorkflowExecutor::ordered_steps(&def);
        let plan = serde_json::json!({
            "workflow_id": def.id,
            "workflow_version": def.version,
            "run_id": run.id,
            "steps": steps.iter().map(|s| serde_json::json!({
                "id": s.id,
                "kind": s.kind.to_string(),
                "reference": s.reference,
                "description": s.description,
                "dependencies": s.dependencies,
                "required_capabilities": s.required_capabilities.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                "verification_required": s.verification_required,
            })).collect::<Vec<_>>(),
            "verification": def.verification,
        });
        self.emit(
            EventType::WorkflowStarted,
            serde_json::json!({"workflow_id": def.id, "run_id": run.id}),
        );
        self.audit("WORKFLOW_STARTED", &def.id, true, &run.id);
        Ok((run, plan))
    }

    pub fn complete_run(&self, run_id: &str, outputs: serde_json::Value, success: bool) {
        if let Some(mut run) = self.registry.get_run(run_id) {
            run.status = if success {
                WorkflowRunStatus::Completed
            } else {
                WorkflowRunStatus::Failed
            };
            run.outputs = Some(outputs.clone());
            run.updated_at = now_millis();
            let _ = self.registry.record_run(&run);
            self.emit(
                if success {
                    EventType::WorkflowCompleted
                } else {
                    EventType::WorkflowFailed
                },
                serde_json::json!({"run_id": run_id, "workflow_id": run.workflow_id}),
            );
        }
    }

    pub fn get_run(&self, run_id: &str) -> Option<WorkflowRun> {
        self.registry.get_run(run_id)
    }
}
