use crate::privacy::PrivacyClassification;
use crate::skills::types::SkillSource;
use crate::workflows::types::*;
use crate::workflows::validator::validate_workflow;

fn sample_wf() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "wf-test".to_string(),
        name: "Test".to_string(),
        version: "1.0.0".to_string(),
        description: "safe test workflow".to_string(),
        source: SkillSource::User,
        status: WorkflowStatus::Draft,
        classification: PrivacyClassification::Internal,
        inputs: serde_json::json!({"type":"object"}),
        steps: vec![
            WorkflowStep {
                id: "a".to_string(),
                kind: WorkflowStepKind::FileOperation,
                reference: "workspace://a.md".to_string(),
                description: "read a".to_string(),
                dependencies: vec![],
                required_capabilities: vec![],
                inputs: None,
                expected_output: None,
                verification_required: false,
            },
            WorkflowStep {
                id: "b".to_string(),
                kind: WorkflowStepKind::Verification,
                reference: "output_schema".to_string(),
                description: "verify".to_string(),
                dependencies: vec!["a".to_string()],
                required_capabilities: vec![],
                inputs: None,
                expected_output: None,
                verification_required: true,
            },
        ],
        outputs: serde_json::json!({"type":"object"}),
        verification: vec!["output_schema".to_string()],
        policy_requirements: vec![],
        created_at: now_millis(),
        updated_at: now_millis(),
    }
}

#[test]
fn test_workflow_valid() {
    assert!(validate_workflow(&sample_wf()).is_ok());
}

#[test]
fn test_workflow_cycle_rejected() {
    let mut wf = sample_wf();
    wf.steps[0].dependencies = vec!["b".to_string()];
    assert!(validate_workflow(&wf).is_err());
}

#[test]
fn test_workflow_unknown_dep_rejected() {
    let mut wf = sample_wf();
    wf.steps[1].dependencies = vec!["missing".to_string()];
    assert!(validate_workflow(&wf).is_err());
}

#[test]
fn test_workflow_order_respects_deps() {
    let wf = sample_wf();
    let ordered = crate::workflows::executor::WorkflowExecutor::ordered_steps(&wf);
    assert_eq!(ordered.len(), 2);
    assert_eq!(ordered[0].id, "a");
    assert_eq!(ordered[1].id, "b");
}

#[test]
fn test_workflow_unauthorized_tool_blocked() {
    // Adversarial D: workflow references an unregistered tool -> fail closed.
    use crate::db::{DatabaseManager, DbConfig};
    use std::sync::Arc;
    let db = Arc::new(DatabaseManager::new(DbConfig::in_memory()));
    let _ = db.initialize();
    let wf_registry = crate::workflows::registry::WorkflowRegistry::new(db.clone());
    let skill_registry = crate::skills::registry::SkillRegistry::new(db);
    let tool_registry = crate::tools::ToolRegistry::new();
    let mut wf = sample_wf();
    wf.id = "wf-evil".to_string();
    wf.status = WorkflowStatus::Active;
    wf.steps.push(WorkflowStep {
        id: "evil".to_string(),
        kind: WorkflowStepKind::Tool,
        reference: "ghost.tool".to_string(),
        description: "evil tool".to_string(),
        dependencies: vec!["b".to_string()],
        required_capabilities: vec![],
        inputs: None,
        expected_output: None,
        verification_required: false,
    });
    let stored = wf_registry.register(wf).unwrap();
    assert_eq!(stored.status, WorkflowStatus::Active);
    let res = crate::workflows::executor::WorkflowExecutor::authorize(
        &wf_registry,
        &skill_registry,
        &tool_registry,
        "wf-evil",
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_workflow_version_pinned_run() {
    use crate::db::{DatabaseManager, DbConfig};
    use std::sync::Arc;
    let db = Arc::new(DatabaseManager::new(DbConfig::in_memory()));
    let _ = db.initialize();
    let wf_registry = crate::workflows::registry::WorkflowRegistry::new(db);
    let wf = sample_wf();
    let stored = wf_registry.register(wf).unwrap();
    let run = crate::workflows::executor::WorkflowExecutor::create_run(
        &wf_registry,
        &stored,
        Some("task-1".to_string()),
        None,
        None,
        serde_json::json!({}),
    )
    .unwrap();
    // Exact version pinned at task start; later definition changes cannot mutate it.
    assert_eq!(run.workflow_version, stored.version);
    let fetched = wf_registry.get_run(&run.id).unwrap();
    assert_eq!(fetched.workflow_id, stored.id);
}
