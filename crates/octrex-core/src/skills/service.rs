use crate::db::manager::DatabaseManager;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::skills::errors::SkillError;
use crate::skills::matcher::{RankedSkill, SkillMatchRequest, SkillMatcher};
use crate::skills::registry::SkillRegistry;
use crate::skills::types::{now_millis, SkillDefinition, SkillSource, SkillStatus};
use crate::skills::{executor::SkillExecutor, validator};
use std::sync::Arc;

/// Free/local model helper: prefer an authorized local/free model through the
/// existing ModelRouter path (ModelRegistry filtered to local/free). Never
/// calls paid APIs directly and never falls back to cloud automatically.
pub fn preferred_free_local_model(model_registry: &crate::models::ModelRegistry) -> Option<String> {
    let models = model_registry.list_models();
    // Prefer local execution-mode models first.
    for m in &models {
        if m.execution_mode == crate::providers::ExecutionMode::Local {
            return Some(m.id.clone());
        }
    }
    // Then the free router alias if present.
    for m in &models {
        if m.id.contains("free") || m.provider_id.contains("opencode") {
            return Some(m.id.clone());
        }
    }
    None
}

/// High-level skill service: registry + validation + matching + audit/events.
/// Capability enforcement stays in Phase 9 ToolRuntime.
#[derive(Clone)]
pub struct SkillService {
    registry: SkillRegistry,
    event_bus: Arc<EventBus>,
    db: Arc<DatabaseManager>,
}

impl SkillService {
    pub fn new(db: Arc<DatabaseManager>, event_bus: Arc<EventBus>) -> Self {
        Self {
            registry: SkillRegistry::new(db.clone()),
            event_bus,
            db,
        }
    }

    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }

    fn emit(&self, event_type: EventType, payload: serde_json::Value) {
        let _ = self
            .event_bus
            .publish(EventEnvelope::new(event_type, payload));
    }

    fn audit(&self, event_type: &str, skill_id: &str, success: bool, reason: &str) {
        let id = format!("audit-{}", uuid::Uuid::new_v4().simple());
        let now = now_millis();
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_records (id, timestamp, event_type, task_id, session_id, workspace_id, actor, provider, model, route, privacy_classification, policy_source, permission, tool, success, reason)
                 VALUES (?1, ?2, ?3, NULL, NULL, NULL, 'skill_service', NULL, NULL, NULL, NULL, NULL, NULL, ?4, ?5, ?6)",
                rusqlite::params![id, now as i64, event_type, skill_id, if success { 1 } else { 0 }, reason],
            )
            .map_err(|e| crate::error::OctrexError::Internal {
                message: e.to_string(),
            })?;
            Ok(())
        });
    }

    pub fn create_skill(&self, def: SkillDefinition) -> Result<SkillDefinition, SkillError> {
        let stored = self.registry.register(def)?;
        self.emit(
            EventType::SkillCreated,
            serde_json::json!({"skill_id": stored.id, "version": stored.version, "source": stored.source.to_string()}),
        );
        self.audit("SKILL_CREATED", &stored.id, true, "created");
        Ok(stored)
    }

    pub fn validate_skill(&self, id: &str) -> Result<bool, SkillError> {
        let def = self.registry.get(id).ok_or_else(|| SkillError::NotFound {
            skill_id: id.to_string(),
        })?;
        match validator::validate_skill(&def) {
            Ok(()) => {
                self.emit(
                    EventType::SkillValidated,
                    serde_json::json!({"skill_id": id, "valid": true}),
                );
                Ok(true)
            }
            Err(e) => {
                self.emit(
                    EventType::SkillValidated,
                    serde_json::json!({"skill_id": id, "valid": false, "reason": e.to_string()}),
                );
                Err(e)
            }
        }
    }

    pub fn enable(&self, id: &str) -> Result<SkillDefinition, SkillError> {
        let def = self.registry.set_status(id, SkillStatus::Active)?;
        self.emit(EventType::SkillEnabled, serde_json::json!({"skill_id": id}));
        self.audit("SKILL_ENABLED", id, true, "enabled");
        Ok(def)
    }

    pub fn disable(&self, id: &str) -> Result<SkillDefinition, SkillError> {
        let def = self.registry.set_status(id, SkillStatus::Disabled)?;
        self.emit(
            EventType::SkillDisabled,
            serde_json::json!({"skill_id": id}),
        );
        self.audit("SKILL_DISABLED", id, true, "disabled");
        Ok(def)
    }

    pub fn approve_imported(
        &self,
        id: &str,
        approver: &str,
    ) -> Result<SkillDefinition, SkillError> {
        let def = self.registry.approve(id, approver)?;
        self.emit(
            EventType::SkillValidated,
            serde_json::json!({"skill_id": id, "approved_by": approver, "status": def.status.to_string()}),
        );
        self.audit("SKILL_APPROVED", id, true, approver);
        Ok(def)
    }

    pub fn match_skills(&self, req: &SkillMatchRequest) -> Vec<RankedSkill> {
        SkillMatcher::match_skills(&self.registry, req)
    }

    /// Authorize execution and pin exact version. Delegates actual capability
    /// checks to ToolRuntime at run time.
    pub fn authorize_execution(
        &self,
        skill_id: &str,
        pinned_version: Option<&str>,
        tool_registry: &crate::tools::ToolRegistry,
    ) -> Result<SkillDefinition, SkillError> {
        SkillExecutor::authorize_execution(&self.registry, skill_id, pinned_version, tool_registry)
    }

    /// Seed a minimal set of safe built-in skills (idempotent).
    pub fn seed_builtins(&self) {
        let builtins = vec![
            (
                "report-draft",
                "Draft Report",
                "Draft a structured report from workspace files without exfiltration.",
            ),
            (
                "file-summary",
                "Summarize Files",
                "Summarize workspace files into a concise brief.",
            ),
            (
                "verification-check",
                "Verification Check",
                "Run read-only verification checks over task outputs.",
            ),
        ];
        for (id, name, desc) in builtins {
            if self.registry.get(id).is_some() {
                continue;
            }
            let def = SkillDefinition {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                version: "1.0.0".to_string(),
                owner: "system".to_string(),
                source: SkillSource::BuiltIn,
                status: SkillStatus::Draft,
                classification: crate::privacy::PrivacyClassification::Internal,
                capabilities_required: vec![crate::tools::ToolCapability::FilesystemRead],
                allowed_tools: vec!["fs.read".to_string()],
                workflow: vec![crate::skills::types::SkillStep {
                    id: "step-1".to_string(),
                    kind: crate::skills::types::SkillStepKind::FileOperation,
                    reference: "workspace://relative/path".to_string(),
                    description: "Read workspace files".to_string(),
                    required_capabilities: vec![crate::tools::ToolCapability::FilesystemRead],
                    inputs: None,
                    expected_output: Some("file contents summary".to_string()),
                }],
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({"type": "object"}),
                verification_requirements: vec!["output_schema".to_string()],
                provenance: crate::skills::types::SkillProvenance {
                    created_by: "system".to_string(),
                    source: SkillSource::BuiltIn,
                    imported_from: None,
                    approval_status: "APPROVED".to_string(),
                    validation_status: "VALID".to_string(),
                    tasks_used_in: Vec::new(),
                    last_updated: now_millis(),
                },
                created_at: now_millis(),
                updated_at: now_millis(),
            };
            let _ = self.registry.register(def);
        }
    }
}
