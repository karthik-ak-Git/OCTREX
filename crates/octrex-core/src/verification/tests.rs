use crate::app::ApplicationState;
use crate::verification::{
    CommandEvidenceInput, CommandKind, StepEvidenceInput, TaskConstraint, TaskVerificationRequest,
    ToolEvidenceInput, UserRequirement, VerificationStatus,
};

fn test_state() -> ApplicationState {
    ApplicationState::initialize()
}

fn local_only_task(task_id: &str) -> TaskVerificationRequest {
    let mut req = TaskVerificationRequest::for_task(task_id);
    req.constraints = vec![TaskConstraint {
        constraint_type: "local_only".to_string(),
        description: "Keep everything local".to_string(),
        observed: Some("local_only".to_string()),
    }];
    req
}

#[test]
fn test_model_claim_alone_is_not_verification() {
    let state = test_state();
    let mut req = TaskVerificationRequest::for_task("task-claim-only");
    req.model_claim = Some("Done. Everything completed successfully.".to_string());
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(
        !result.status.is_pass(),
        "model claim alone must never verify"
    );
    assert_eq!(result.confidence, 0.0);
}

#[test]
fn test_missing_artifact_fails() {
    let state = test_state();
    let mut req = TaskVerificationRequest::for_task("task-missing-artifact");
    req.expected_artifacts = vec![crate::verification::ExpectedArtifact {
        name: "report.pdf".to_string(),
        artifact_type: Some("pdf".to_string()),
        max_size_bytes: None,
    }];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(matches!(
        result.status,
        VerificationStatus::Fail | VerificationStatus::Blocked
    ));
}

#[test]
fn test_command_failure_fails() {
    let state = test_state();
    let mut req = TaskVerificationRequest::for_task("task-cmd-fail");
    req.commands = vec![CommandEvidenceInput {
        command_ref: "cargo test".to_string(),
        kind: CommandKind::Test,
        exit_code: Some(1),
        duration_ms: 120,
        stdout: "failures: 2".to_string(),
        stderr: "".to_string(),
    }];
    // A bare failing command with no other evidence must fail even if the
    // model claims tests passed.
    req.model_claim = Some("tests passed".to_string());
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(!result.status.is_pass());
}

#[test]
fn test_tool_failure_fails() {
    let state = test_state();
    let mut req = TaskVerificationRequest::for_task("task-tool-fail");
    req.tools = vec![ToolEvidenceInput {
        tool_id: "fs.write".to_string(),
        status: "FAILED".to_string(),
        duration_ms: 5,
        result: serde_json::json!({}),
        expected_result_field: None,
    }];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(!result.status.is_pass());
}

#[test]
fn test_schema_mismatch_fails() {
    let state = test_state();
    let mut req = TaskVerificationRequest::for_task("task-schema");
    req.output_schema = Some(serde_json::json!({
        "type": "object",
        "required": ["title", "sections"],
        "properties": { "title": { "type": "string" } }
    }));
    req.output_value = Some(serde_json::json!({ "title": "x" }));
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(!result.status.is_pass());
}

#[test]
fn test_successful_verification_passes() {
    let state = test_state();
    let mut req = local_only_task("task-success");
    req.steps = vec![StepEvidenceInput {
        step_id: "step-1".to_string(),
        status: "COMPLETED".to_string(),
        has_error: false,
    }];
    req.commands = vec![CommandEvidenceInput {
        command_ref: "cargo test".to_string(),
        kind: CommandKind::Test,
        exit_code: Some(0),
        duration_ms: 50,
        stdout: "test result: ok. 3 passed".to_string(),
        stderr: "".to_string(),
    }];
    req.tools = vec![ToolEvidenceInput {
        tool_id: "fs.write".to_string(),
        status: "VALID".to_string(),
        duration_ms: 3,
        result: serde_json::json!({ "bytes_written": 10 }),
        expected_result_field: Some("bytes_written".to_string()),
    }];
    req.output_schema = Some(serde_json::json!({
        "type": "object",
        "required": ["title"],
        "properties": { "title": { "type": "string" } }
    }));
    req.output_value = Some(serde_json::json!({ "title": "Report" }));
    req.requirements = vec![UserRequirement {
        requirement: "Create a report".to_string(),
        requirement_type: "create_report".to_string(),
        observed_satisfied: Some(true),
    }];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(
        result.status.is_pass(),
        "expected pass, got {:?}",
        result.status
    );
    // Completion gate must allow completion.
    let decision = state.verification_engine.completion_gate(&result, false);
    assert_eq!(
        decision,
        crate::verification::CompletionDecision::AllowCompletion
    );
}

#[test]
fn test_unknown_security_state_blocks() {
    let state = test_state();
    // Workspace-scoped file check without a workspace: fail-closed.
    let mut req = TaskVerificationRequest::for_task("task-unknown-fs");
    req.expected_files = vec![crate::verification::ExpectedFile::exists("report.pdf")];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert_eq!(result.status, VerificationStatus::Blocked);
    let decision = state.verification_engine.completion_gate(&result, false);
    assert_eq!(decision, crate::verification::CompletionDecision::Blocked);
}

#[test]
fn test_protected_file_violation_blocks() {
    use crate::workspace::WorkspaceRegistry;
    let state = test_state();
    let tmp = std::env::temp_dir().join(format!("octx-ver-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join(".env"), "SECRET=1").unwrap();
    let ws = state
        .workspace_registry
        .register_workspace("ver-ws", tmp.clone());
    // Persist workspace so FilesystemSecurityService can resolve it.
    {
        use crate::db::repository::{SqliteWorkspaceRepository, WorkspaceRepository};
        let repo = SqliteWorkspaceRepository::new((*state.db).clone());
        let record = crate::workspace::Workspace {
            id: ws.id.clone(),
            name: "ver-ws".to_string(),
            path: tmp.clone(),
            classification: "PUBLIC".to_string(),
            created_at: 0,
            updated_at: 0,
        };
        let _ = repo.create_workspace(&record);
    }
    let mut req = TaskVerificationRequest::for_task("task-protected");
    req.workspace_id = Some(ws.id.to_string());
    req.expected_files = vec![crate::verification::ExpectedFile::exists(".env")];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(
        result.status == VerificationStatus::Blocked || !result.status.is_pass(),
        "protected file probe must not pass, got {:?}",
        result.status
    );
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = WorkspaceRegistry::new();
}

#[test]
fn test_requirement_mismatch_fails() {
    let state = test_state();
    let mut req = TaskVerificationRequest::for_task("task-req-mismatch");
    req.steps = vec![StepEvidenceInput {
        step_id: "s1".to_string(),
        status: "COMPLETED".to_string(),
        has_error: false,
    }];
    req.requirements = vec![UserRequirement {
        requirement: "Use Excel format".to_string(),
        requirement_type: "excel_format".to_string(),
        observed_satisfied: Some(false),
    }];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(!result.status.is_pass());
}

#[test]
fn test_cloud_audit_violates_local_only() {
    let state = test_state();
    // Seed an audit record showing cloud routing for this task.
    {
        use crate::db::repository::{AuditRepository, SqliteAuditRepository};
        let repo = SqliteAuditRepository::new((*state.db).clone());
        let record = crate::db::models::AuditRecord {
            id: format!("audit-{}", uuid::Uuid::new_v4().simple()),
            timestamp: 1,
            event_type: "MODEL_COMPLETED".to_string(),
            task_id: Some(crate::ids::TaskId::from("task-cloud-violation")),
            session_id: None,
            workspace_id: None,
            actor: "model_runtime".to_string(),
            provider: Some("groq".to_string()),
            model: Some("llama-3.3-70b-versatile".to_string()),
            route: Some("cloud".to_string()),
            privacy_classification: None,
            policy_source: None,
            permission: None,
            tool: None,
            success: true,
            reason: None,
        };
        repo.record_audit(&record).unwrap();
    }
    let mut req = local_only_task("task-cloud-violation");
    req.steps = vec![StepEvidenceInput {
        step_id: "s1".to_string(),
        status: "COMPLETED".to_string(),
        has_error: false,
    }];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(
        !result.status.is_pass(),
        "cloud audit must violate local-only"
    );
}

#[test]
fn test_repair_loop_is_bounded() {
    let state = test_state();
    state.verification_engine.set_max_repair_attempts(2);
    assert!(state
        .verification_engine
        .register_repair_attempt("task-repair-bounded")
        .is_ok());
    // Second registration may still be ok depending on persisted runs; force
    // exhaustion by registering up to the cap.
    let mut exhausted = false;
    for _ in 0..5 {
        if state
            .verification_engine
            .register_repair_attempt("task-repair-bounded")
            .is_err()
        {
            exhausted = true;
            break;
        }
    }
    assert!(exhausted, "repair loop must be bounded");
    state.verification_engine.set_max_repair_attempts(3);
}

#[test]
fn test_task_isolation_between_runs() {
    let state = test_state();
    let mut ok_req = local_only_task("task-isolation-a");
    ok_req.steps = vec![StepEvidenceInput {
        step_id: "s1".to_string(),
        status: "COMPLETED".to_string(),
        has_error: false,
    }];
    let ok_result = state.verification_engine.verify_task(&ok_req).unwrap();
    assert!(ok_result.status.is_pass());

    let mut bad_req = TaskVerificationRequest::for_task("task-isolation-b");
    bad_req.model_claim = Some("done".to_string());
    bad_req.steps = vec![StepEvidenceInput {
        step_id: "s2".to_string(),
        status: "FAILED".to_string(),
        has_error: true,
    }];
    let bad_result = state.verification_engine.verify_task(&bad_req).unwrap();
    assert!(!bad_result.status.is_pass());

    let runs_a = state
        .verification_engine
        .list_runs_by_task("task-isolation-a")
        .unwrap();
    let runs_b = state
        .verification_engine
        .list_runs_by_task("task-isolation-b")
        .unwrap();
    assert!(!runs_a.is_empty() && !runs_b.is_empty());
    assert_ne!(runs_a[0].verification_id, runs_b[0].verification_id);
}

#[test]
fn test_artifact_status_stamped_on_verify() {
    use crate::db::models::ArtifactRecord;
    use crate::db::repository::{ArtifactRepository, SqliteArtifactRepository};

    let state = test_state();
    let repo = SqliteArtifactRepository::new((*state.db).clone());
    let task_id = crate::ids::TaskId::from("task-artifact-stamp");

    // The pre-existing artifacts table references tasks(id): seed the parent row.
    {
        use crate::db::repository::{SqliteTaskRepository, TaskRepository};
        let task_repo = SqliteTaskRepository::new((*state.db).clone());
        let mut task = crate::task::Task::new("artifact stamp task", None, None, None);
        task.id = task_id.clone();
        task_repo.create_task(&task).unwrap();
    }

    let good = ArtifactRecord {
        id: crate::ids::ArtifactId::new(),
        task_id: Some(task_id.clone()),
        workspace_id: None,
        name: "report.pdf".to_string(),
        path: "report.pdf".to_string(),
        artifact_type: "pdf".to_string(),
        size: 1024,
        checksum: None,
        created_at: 1,
        verification_status: "UNVERIFIED".to_string(),
    };
    repo.create_artifact(&good).unwrap();

    let empty = ArtifactRecord {
        id: crate::ids::ArtifactId::new(),
        task_id: Some(task_id.clone()),
        workspace_id: None,
        name: "empty.pdf".to_string(),
        path: "empty.pdf".to_string(),
        artifact_type: "pdf".to_string(),
        size: 0,
        checksum: None,
        created_at: 1,
        verification_status: "UNVERIFIED".to_string(),
    };
    repo.create_artifact(&empty).unwrap();

    let mut req = TaskVerificationRequest::for_task("task-artifact-stamp");
    req.expected_artifacts = vec![
        crate::verification::ExpectedArtifact {
            name: "report.pdf".to_string(),
            artifact_type: None,
            max_size_bytes: None,
        },
        crate::verification::ExpectedArtifact {
            name: "empty.pdf".to_string(),
            artifact_type: None,
            max_size_bytes: None,
        },
    ];
    let result = state.verification_engine.verify_task(&req).unwrap();
    assert!(!result.status.is_pass(), "empty artifact must fail the run");

    let good_after = repo.get_artifact(&good.id).unwrap().unwrap();
    assert_eq!(good_after.verification_status, "VERIFIED");
    let empty_after = repo.get_artifact(&empty.id).unwrap().unwrap();
    assert_eq!(empty_after.verification_status, "FAILED");
}
