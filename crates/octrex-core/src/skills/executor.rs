use crate::skills::errors::SkillError;
use crate::skills::policy;
use crate::skills::registry::SkillRegistry;
use crate::skills::types::SkillDefinition;
use crate::tools::{ToolId, ToolRegistry};

/// Skill execution pre-checks. Actual capability grants remain authoritative
/// in ToolRuntime / Filesystem / Network / Permission boundaries; the skill
/// declaration is never treated as authorization.
pub struct SkillExecutor;

impl SkillExecutor {
    /// Re-validate against current policy immediately before execution and
    /// pin the exact skill version for the task.
    pub fn authorize_execution(
        registry: &SkillRegistry,
        skill_id: &str,
        pinned_version: Option<&str>,
        tool_registry: &ToolRegistry,
    ) -> Result<SkillDefinition, SkillError> {
        let def = if let Some(v) = pinned_version {
            registry
                .get_version(skill_id, v)
                .ok_or_else(|| SkillError::NotFound {
                    skill_id: format!("{}@{}", skill_id, v),
                })?
        } else {
            registry.get(skill_id).ok_or_else(|| SkillError::NotFound {
                skill_id: skill_id.to_string(),
            })?
        };

        // Selection != authorization: validate again against current policy.
        policy::check_eligibility(&def)?;
        policy::check_policy_compatibility(&def)?;
        crate::skills::security::SkillSecurity::scan_for_injection(&def)?;

        // Every referenced tool must exist; unknown tools fail closed.
        for step in &def.workflow {
            if matches!(step.kind, crate::skills::types::SkillStepKind::Tool) {
                // Allowlist check first.
                if !def.allowed_tools.contains(&step.reference) {
                    return Err(SkillError::CapabilityDenied {
                        reason: format!(
                            "Skill step '{}' references tool '{}' outside skill allowlist",
                            step.id, step.reference
                        ),
                    });
                }
                let tid = ToolId::new(&step.reference);
                if tool_registry.get(&tid).is_none() {
                    return Err(SkillError::CapabilityDenied {
                        reason: format!("Skill references unregistered tool '{}'", step.reference),
                    });
                }
            }
        }

        Ok(def)
    }

    /// Build a deterministic execution plan (Phase 11 Orchestrator consumes
    /// this; this executor never implements its own orchestration loop).
    pub fn build_execution_plan(def: &SkillDefinition, task_id: &str) -> serde_json::Value {
        serde_json::json!({
            "skill_id": def.id,
            "skill_version": def.version,
            "versioned_id": def.versioned_id(),
            "task_id": task_id,
            "steps": def.workflow.iter().map(|s| serde_json::json!({
                "id": s.id,
                "kind": s.kind.to_string(),
                "reference": s.reference,
                "description": s.description,
                "required_capabilities": s.required_capabilities.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                "inputs": s.inputs,
                "expected_output": s.expected_output,
            })).collect::<Vec<_>>(),
            "verification_requirements": def.verification_requirements,
            "pinned_at": crate::skills::types::now_millis(),
        })
    }
}
