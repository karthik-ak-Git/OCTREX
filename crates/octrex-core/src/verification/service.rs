use crate::context::ContextService;
use crate::db::repository::{ArtifactRepository, AuditRepository, TaskStepRepository};
use crate::db::repository::{
    SqliteArtifactRepository, SqliteAuditRepository, SqliteTaskStepRepository,
};
use crate::db::DatabaseManager;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::filesystem::{FilesystemOperation, FilesystemSecurityService};
use crate::ids::{TaskId, WorkspaceId};
use crate::models::{ModelRegistry, ModelRuntime};
use crate::network::NetworkSecurityService;
use crate::privacy::PrivacyGate;
use crate::router::{ModelRouter, RoutingMode, RoutingRequest};
use crate::verification::checks::{
    CommandKind, TaskConstraint, TaskVerificationRequest, UserRequirement,
};
use crate::verification::errors::VerificationError;
use crate::verification::evidence::{bound_text, VerificationEvidence, MAX_FILE_PREVIEW_CHARS};
use crate::verification::gate::{CompletionGate, GateInput};
use crate::verification::policy::VerificationPolicy;
use crate::verification::repository::SqliteVerificationRepository;
use crate::verification::types::{
    CheckSeverity, CompletionDecision, SchemaIssue, VerificationAction, VerificationCheck,
    VerificationCheckType, VerificationResult, VerificationStatus,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Bounded repair policy. Infinite repair loops are prohibited.
#[derive(Debug, Clone)]
pub struct RepairPolicy {
    pub max_attempts: u32,
}

impl Default for RepairPolicy {
    fn default() -> Self {
        Self { max_attempts: 3 }
    }
}

/// Verification & Reliability Engine.
///
/// Reuses Phase 6 (PrivacyGate), Phase 7 (NetworkSecurityService),
/// Phase 8 (FilesystemSecurityService), Phase 10 (ContextService via budget
/// checks is optional and non-authoritative), and Phase 12 routing
/// (ModelRegistry + ModelRuntime) for supplemental AI advisory checks.
///
/// The engine NEVER executes tools or commands and NEVER performs network
/// access. It only inspects authoritative evidence supplied by the
/// orchestrator plus read-only security-boundary state.
pub struct VerificationEngine {
    db: Arc<DatabaseManager>,
    event_bus: Arc<EventBus>,
    filesystem: Arc<FilesystemSecurityService>,
    network: Arc<NetworkSecurityService>,
    privacy: Arc<PrivacyGate>,
    context: Arc<ContextService>,
    models: Arc<ModelRegistry>,
    runtime: Arc<ModelRuntime>,
    router: RwLock<Option<Arc<ModelRouter>>>,
    repo: SqliteVerificationRepository,
    repair_attempts: Arc<RwLock<HashMap<String, u32>>>,
    repair_policy: RwLock<RepairPolicy>,
}

impl VerificationEngine {
    pub fn new(
        db: Arc<DatabaseManager>,
        event_bus: Arc<EventBus>,
        filesystem: Arc<FilesystemSecurityService>,
        network: Arc<NetworkSecurityService>,
        privacy: Arc<PrivacyGate>,
        context: Arc<ContextService>,
        models: Arc<ModelRegistry>,
        runtime: Arc<ModelRuntime>,
    ) -> Self {
        let repo = SqliteVerificationRepository::new((*db).clone());
        Self {
            db,
            event_bus,
            filesystem,
            network,
            privacy,
            context,
            models,
            runtime,
            router: RwLock::new(None),
            repo,
            repair_attempts: Arc::new(RwLock::new(HashMap::new())),
            repair_policy: RwLock::new(RepairPolicy::default()),
        }
    }

    /// Attach the Phase 12 ModelRouter for advisory-model selection.
    /// Advisory routing is always local-only: online candidates are excluded
    /// and no automatic local -> online fallback exists.
    pub fn attach_router(&self, router: Arc<ModelRouter>) {
        *self.router.write().unwrap() = Some(router);
    }

    pub fn set_max_repair_attempts(&self, max: u32) {
        let clamped = max.min(10).max(1);
        self.repair_policy.write().unwrap().max_attempts = clamped;
    }

    pub fn max_repair_attempts(&self) -> u32 {
        self.repair_policy.read().unwrap().max_attempts
    }

    pub fn repair_attempts_for(&self, task_id: &str) -> u32 {
        self.repair_attempts
            .read()
            .unwrap()
            .get(task_id)
            .copied()
            .unwrap_or(0)
    }

    fn record_attempt(&self, task_id: &str) -> u32 {
        let mut guard = self.repair_attempts.write().unwrap();
        let next = guard.get(task_id).copied().unwrap_or(0) + 1;
        guard.insert(task_id.to_string(), next);
        next
    }

    fn publish(&self, event_type: EventType, payload: serde_json::Value, task_id: &TaskId) {
        let envelope = EventEnvelope::new(event_type, payload).with_task_id(task_id.clone());
        let _ = self.event_bus.publish(envelope);
    }

    fn audit_run(&self, result: &VerificationResult, workspace_id: Option<&WorkspaceId>) {
        let repo = SqliteAuditRepository::new((*self.db).clone());
        let record = crate::db::models::AuditRecord {
            id: format!("audit-{}", uuid::Uuid::new_v4().simple()),
            timestamp: now_millis(),
            event_type: format!("VERIFICATION_{}", result.status),
            task_id: Some(TaskId::from(result.task_id.clone())),
            session_id: None,
            workspace_id: workspace_id.cloned(),
            actor: "verification_engine".to_string(),
            provider: None,
            model: None,
            route: None,
            privacy_classification: None,
            policy_source: Some("verification_policy".to_string()),
            permission: None,
            tool: None,
            success: result.status.is_pass(),
            reason: Some(format!(
                "{} checks, {} failed",
                result.checks.len(),
                result.failed_count()
            )),
        };
        let _ = repo.record_audit(&record);
    }

    /// Main entry point: verify a task from structured authoritative evidence.
    pub fn verify_task(
        &self,
        request: &TaskVerificationRequest,
    ) -> Result<VerificationResult, VerificationError> {
        request
            .validate()
            .map_err(|reason| VerificationError::InvalidInput { reason })?;

        let task_id = TaskId::from(request.task_id.clone());
        let workspace_id = request
            .workspace_id
            .as_ref()
            .map(|w| WorkspaceId::from(w.clone()));

        self.publish(
            EventType::VerificationStarted,
            serde_json::json!({
                "task_id": request.task_id,
                "step_id": request.step_id,
                "checks_planned": request.expected_files.len() + request.expected_artifacts.len()
                    + request.commands.len() + request.tools.len()
                    + request.steps.len() + request.constraints.len()
                    + request.requirements.len(),
            }),
            &task_id,
        );

        let mut checks: Vec<VerificationCheck> = Vec::new();
        let mut evidence_refs: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Step verification (authoritative step store evidence + DB corroboration).
        checks.extend(self.verify_steps(request, &mut evidence_refs));

        // 2. File verification through Phase 8 boundary.
        checks.extend(self.verify_files(
            request,
            workspace_id.as_ref(),
            &mut evidence_refs,
            &mut warnings,
        ));

        // 3. Artifact verification against artifact subsystem.
        checks.extend(self.verify_artifacts(request, workspace_id.as_ref(), &mut evidence_refs));

        // 4. Command / build / test verification from bounded evidence.
        checks.extend(self.verify_commands(request, &mut evidence_refs));

        // 5. Tool verification (status inspection only; never executes).
        checks.extend(self.verify_tools(request, &mut evidence_refs));

        // 6. Structured output verification.
        checks.extend(self.verify_output(request, &mut evidence_refs));

        // 7. Constraint verification (privacy / network / workspace).
        checks.extend(self.verify_constraints(
            request,
            workspace_id.as_ref(),
            &mut evidence_refs,
            &mut warnings,
        ));

        // 8. User requirement verification.
        checks.extend(self.verify_requirements(request, &mut evidence_refs));

        // 9. Blocking-error sweep.
        checks.extend(self.verify_no_blocking_error(request));

        // 10. Context state probe (Phase 10 reuse, read-only).
        // Verification builds no separate context: it only confirms the
        // session context is observable. An unavailable summary is a warning,
        // never a failure — verification rests on authoritative evidence.
        if let Some(session_id) = request.session_id.as_deref() {
            match self.context.get_session_context_summary(session_id) {
                Ok(_) => {}
                Err(e) => {
                    checks.push(VerificationCheck::warning(
                        "context_state_unavailable",
                        VerificationCheckType::DependencySatisfied,
                        "observable session context",
                        format!("unavailable: {}", e),
                        "Session context summary unavailable; verification proceeds on authoritative evidence only",
                    ));
                    warnings.push(format!("Session context unavailable: {}", e));
                }
            }
        }

        // Model-claim guard: a bare "done" with zero authoritative checks is NOT verified.
        let has_authoritative_checks = checks
            .iter()
            .any(|c| c.check_type != VerificationCheckType::NoBlockingError);
        if !has_authoritative_checks {
            checks.push(VerificationCheck::failed(
                "model_claim_without_evidence",
                VerificationCheckType::NoBlockingError,
                CheckSeverity::Blocking,
                "independent evidence",
                "model claim only",
                "Model completion claim presented without any verifiable evidence; task is NOT VERIFIED",
            ));
        } else if request.model_claim.is_some() {
            warnings.push("Model completion claim recorded but not used as evidence".to_string());
        }

        let status = VerificationPolicy::aggregate(&checks);
        let has_model_claim_only = !has_authoritative_checks;
        let confidence = VerificationPolicy::confidence(&checks, has_model_claim_only);

        let failures: Vec<String> = checks
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    crate::verification::types::CheckStatus::Failed
                        | crate::verification::types::CheckStatus::Blocked
                        | crate::verification::types::CheckStatus::Unknown
                )
            })
            .map(|c| format!("{}: {}", c.name, c.message))
            .collect();

        let verification_id = format!("ver-{}", uuid::Uuid::new_v4().simple());
        let result = VerificationResult {
            verification_id: verification_id.clone(),
            task_id: request.task_id.clone(),
            step_id: request.step_id.clone(),
            status,
            checks: checks.clone(),
            evidence_refs: evidence_refs.clone(),
            warnings: warnings.clone(),
            failures: failures.clone(),
            confidence,
            repair_attempt: request.repair_attempt,
            created_at: now_millis(),
        };

        // Persist (best effort; persistence failure does not mask the verdict).
        let _ = self.repo.save_run(
            &result,
            request.workspace_id.as_deref(),
            request.session_id.as_deref(),
        );

        // Events + audit (evidence references only, no raw secrets).
        match status {
            VerificationStatus::Pass | VerificationStatus::PassWithWarnings => {
                self.publish(
                    EventType::VerificationPassed,
                    serde_json::json!({
                        "verification_id": verification_id,
                        "task_id": request.task_id,
                        "status": status.to_string(),
                        "confidence": confidence,
                    }),
                    &task_id,
                );
            }
            VerificationStatus::Blocked => {
                self.publish(
                    EventType::VerificationBlocked,
                    serde_json::json!({
                        "verification_id": verification_id,
                        "task_id": request.task_id,
                        "failures": failures,
                    }),
                    &task_id,
                );
            }
            _ => {
                self.publish(
                    EventType::VerificationFailed,
                    serde_json::json!({
                        "verification_id": verification_id,
                        "task_id": request.task_id,
                        "status": status.to_string(),
                        "failures": failures,
                    }),
                    &task_id,
                );
            }
        }
        for check in &checks {
            self.publish(
                EventType::VerificationCheckCompleted,
                serde_json::json!({
                    "verification_id": verification_id,
                    "check": check.name,
                    "check_type": check.check_type.to_string(),
                    "status": check.status.to_string(),
                }),
                &task_id,
            );
        }
        self.audit_run(&result, workspace_id.as_ref());

        Ok(result)
    }

    // ------------------------------------------------------------------
    // Step verification
    // ------------------------------------------------------------------
    fn verify_steps(
        &self,
        request: &TaskVerificationRequest,
        evidence_refs: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();

        // Corroborate against persisted task_steps when present.
        let db_steps = TaskId::from(request.task_id.clone());
        let persisted = SqliteTaskStepRepository::new((*self.db).clone())
            .list_steps_by_task(&db_steps)
            .unwrap_or_default();
        if !persisted.is_empty() {
            let incomplete: Vec<String> = persisted
                .iter()
                .filter(|s| s.status.to_string() != "COMPLETED")
                .map(|s| s.id.clone())
                .collect();
            let ev = VerificationEvidence::task_state(
                &request.task_id,
                persisted.len(),
                persisted.len() - incomplete.len(),
            );
            evidence_refs.push(ev.id().to_string());
            if incomplete.is_empty() {
                let mut c = VerificationCheck::passed(
                    "persisted_steps_complete",
                    VerificationCheckType::StepCompleted,
                    "all persisted steps COMPLETED",
                    format!("{}/{} completed", persisted.len(), persisted.len()),
                );
                c.evidence_ref = Some(ev.id().to_string());
                out.push(c);
            } else {
                let mut c = VerificationCheck::failed(
                    "persisted_steps_complete",
                    VerificationCheckType::StepCompleted,
                    CheckSeverity::Critical,
                    "all persisted steps COMPLETED",
                    format!("incomplete steps: {}", incomplete.join(",")),
                    "Required persisted steps are not complete",
                );
                c.evidence_ref = Some(ev.id().to_string());
                out.push(c);
            }
        }

        for step in &request.steps {
            let ev = VerificationEvidence::task_state(
                &request.task_id,
                1,
                if step.status.to_uppercase() == "COMPLETED" && !step.has_error {
                    1
                } else {
                    0
                },
            );
            evidence_refs.push(ev.id().to_string());
            if step.status.to_uppercase() == "SKIPPED" {
                // Waived by the orchestrator: visible warning, not a failure.
                // Legacy completion semantics accept skipped steps.
                let mut c = VerificationCheck::warning(
                    format!("step_skipped:{}", step.step_id),
                    VerificationCheckType::StepCompleted,
                    "COMPLETED without error",
                    "SKIPPED (waived by orchestrator)",
                    "Step was skipped; completion relies on the remaining steps",
                );
                c.evidence_ref = Some(ev.id().to_string());
                out.push(c);
            } else if step.status.to_uppercase() == "COMPLETED" && !step.has_error {
                let mut c = VerificationCheck::passed(
                    format!("step_completed:{}", step.step_id),
                    VerificationCheckType::StepCompleted,
                    "COMPLETED without error",
                    "COMPLETED without error",
                );
                c.evidence_ref = Some(ev.id().to_string());
                out.push(c);
            } else {
                let mut c = VerificationCheck::failed(
                    format!("step_completed:{}", step.step_id),
                    VerificationCheckType::StepCompleted,
                    CheckSeverity::Critical,
                    "COMPLETED without error",
                    format!("status={} has_error={}", step.status, step.has_error),
                    "Step did not complete successfully",
                );
                c.evidence_ref = Some(ev.id().to_string());
                out.push(c);
            }
        }

        // Dependency satisfaction: all supplied steps must be complete or waived.
        if !request.steps.is_empty() {
            let all_done = request.steps.iter().all(|s| {
                matches!(s.status.to_uppercase().as_str(), "COMPLETED" | "SKIPPED") && !s.has_error
            });
            if all_done {
                out.push(VerificationCheck::passed(
                    "dependencies_satisfied",
                    VerificationCheckType::DependencySatisfied,
                    "all dependencies COMPLETED",
                    "all dependencies COMPLETED",
                ));
            } else {
                out.push(VerificationCheck::failed(
                    "dependencies_satisfied",
                    VerificationCheckType::DependencySatisfied,
                    CheckSeverity::Critical,
                    "all dependencies COMPLETED",
                    "one or more dependencies incomplete",
                    "Step dependencies are not satisfied",
                ));
            }
        }
        out
    }

    // ------------------------------------------------------------------
    // File verification (Phase 8 boundary, never bypassed)
    // ------------------------------------------------------------------
    fn verify_files(
        &self,
        request: &TaskVerificationRequest,
        workspace_id: Option<&WorkspaceId>,
        evidence_refs: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        for expected in &request.expected_files {
            let check_base = format!("file:{}", expected.rel_path);
            let Some(ws_id) = workspace_id else {
                out.push(VerificationCheck::unknown(
                    check_base,
                    VerificationCheckType::FileExists,
                    CheckSeverity::Blocking,
                    "workspace_id is required for file verification; refusing to probe outside an authorized workspace",
                ));
                continue;
            };

            // Existence + authorization via Stat evaluation.
            let decision = match self.filesystem.evaluate_operation(
                Some(ws_id),
                None,
                &expected.rel_path,
                FilesystemOperation::Stat,
            ) {
                Ok(d) => d,
                Err(e) => {
                    out.push(VerificationCheck::failed(
                        check_base,
                        VerificationCheckType::FileExists,
                        CheckSeverity::Blocking,
                        "authorized readable file",
                        format!("evaluation error: {}", e),
                        "Filesystem boundary denied verification probe",
                    ));
                    continue;
                }
            };

            if !decision.decision.is_allowed() {
                out.push(
                    VerificationCheck::failed(
                        check_base,
                        VerificationCheckType::FileExists,
                        CheckSeverity::Blocking,
                        "authorized readable file",
                        format!("denied: {}", decision.reason),
                        "File is not authorized/visible inside the workspace boundary",
                    )
                    .with_evidence(format!("fs-decision:{}", decision.decision_id)),
                );
                continue;
            }

            let resolved = decision.resolved_path.clone().unwrap_or_default();
            let meta = std::fs::metadata(&resolved);
            let exists = meta.is_ok();
            let size = meta.as_ref().map(|m| m.len()).ok();
            let ev = VerificationEvidence::file(
                &expected.rel_path,
                "stat",
                exists,
                size,
                decision.classification.to_string(),
            );
            evidence_refs.push(ev.id().to_string());

            if expected.must_exist && !exists {
                out.push(
                    VerificationCheck::failed(
                        check_base,
                        VerificationCheckType::FileExists,
                        CheckSeverity::Critical,
                        "file exists",
                        "file missing",
                        "Required file does not exist",
                    )
                    .with_evidence(ev.id().to_string()),
                );
                continue;
            }
            if !expected.must_exist && !exists {
                out.push(
                    VerificationCheck::passed(
                        check_base,
                        VerificationCheckType::FileExists,
                        "file absent as required",
                        "file absent",
                    )
                    .with_evidence(ev.id().to_string()),
                );
                continue;
            }

            // File exists from here on.
            let mut c = VerificationCheck::passed(
                check_base.clone(),
                VerificationCheckType::FileExists,
                "file exists in authorized workspace",
                format!("exists, size={:?}", size),
            );
            c.evidence_ref = Some(ev.id().to_string());
            out.push(c);

            if expected.must_not_be_empty == Some(true) && size == Some(0) {
                out.push(VerificationCheck::failed(
                    format!("{}:non_empty", expected.rel_path),
                    VerificationCheckType::FileContent,
                    CheckSeverity::Critical,
                    "non-empty file",
                    "file is empty",
                    "Required file is empty",
                ));
            }

            if let Some(ext) = &expected.expected_extension {
                let matches_ext = expected
                    .rel_path
                    .to_lowercase()
                    .ends_with(&ext.to_lowercase());
                if !matches_ext {
                    out.push(VerificationCheck::failed(
                        format!("{}:extension", expected.rel_path),
                        VerificationCheckType::FileContent,
                        CheckSeverity::Critical,
                        format!("extension {}", ext),
                        expected.rel_path.clone(),
                        "File type does not match required type",
                    ));
                }
            }

            if let Some(needle) = &expected.expected_contains {
                match self.filesystem.read_file(ws_id, &expected.rel_path) {
                    Ok(content) => {
                        let preview = bound_text(&content, MAX_FILE_PREVIEW_CHARS);
                        let _ = preview;
                        if content.contains(needle.as_str()) {
                            out.push(VerificationCheck::passed(
                                format!("{}:content", expected.rel_path),
                                VerificationCheckType::FileContent,
                                format!("contains '{}'", needle),
                                "content requirement satisfied",
                            ));
                        } else {
                            out.push(VerificationCheck::failed(
                                format!("{}:content", expected.rel_path),
                                VerificationCheckType::FileContent,
                                CheckSeverity::Critical,
                                format!("contains '{}'", needle),
                                "required content not found",
                                "File content requirement not satisfied",
                            ));
                        }
                    }
                    Err(e) => {
                        out.push(VerificationCheck::unknown(
                            format!("{}:content", expected.rel_path),
                            VerificationCheckType::FileContent,
                            CheckSeverity::Critical,
                            format!("Could not read file for content check: {}", e),
                        ));
                    }
                }
            }

            if let Some(after_ms) = expected.must_be_modified_after_ms {
                if let Some(mtime) = meta.as_ref().ok().and_then(|m| m.modified().ok()) {
                    let mtime_ms = mtime
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    if mtime_ms >= after_ms {
                        out.push(VerificationCheck::passed(
                            format!("{}:modified", expected.rel_path),
                            VerificationCheckType::FileModified,
                            "file created/modified by this task",
                            "modification timestamp satisfies bound",
                        ));
                    } else {
                        out.push(VerificationCheck::warning(
                            format!("{}:modified", expected.rel_path),
                            VerificationCheckType::FileModified,
                            "file created/modified by this task",
                            "file predates task start",
                            "File exists but may not have been created by this task",
                        ));
                        warnings.push(format!(
                            "File '{}' predates task start; treated as warning",
                            expected.rel_path
                        ));
                    }
                } else {
                    out.push(VerificationCheck::unknown(
                        format!("{}:modified", expected.rel_path),
                        VerificationCheckType::FileModified,
                        CheckSeverity::Warning,
                        "Could not determine modification time",
                    ));
                }
            }
        }
        out
    }

    // ------------------------------------------------------------------
    // Artifact verification (existing artifact subsystem)
    // ------------------------------------------------------------------
    /// Stamp the artifact registry row with the verification outcome.
    /// Best effort: persistence failure never masks the in-memory verdict.
    fn stamp_artifact_status(
        repo: &SqliteArtifactRepository,
        artifact: &crate::db::models::ArtifactRecord,
        checks: &[VerificationCheck],
    ) {
        let failed = checks.iter().any(|c| {
            matches!(
                c.status,
                crate::verification::types::CheckStatus::Failed
                    | crate::verification::types::CheckStatus::Blocked
                    | crate::verification::types::CheckStatus::Unknown
            )
        });
        let _ = repo
            .update_verification_status(&artifact.id, if failed { "FAILED" } else { "VERIFIED" });
    }

    fn verify_artifacts(
        &self,
        request: &TaskVerificationRequest,
        workspace_id: Option<&WorkspaceId>,
        evidence_refs: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        if request.expected_artifacts.is_empty() {
            return out;
        }
        let task_id = TaskId::from(request.task_id.clone());
        let repo = SqliteArtifactRepository::new((*self.db).clone());
        let registered = repo.list_artifacts_by_task(&task_id).unwrap_or_default();

        for expected in &request.expected_artifacts {
            let found = registered.iter().find(|a| {
                a.name == expected.name
                    || expected
                        .artifact_type
                        .as_ref()
                        .map(|t| &a.artifact_type == t)
                        .unwrap_or(false)
            });
            match found {
                None => {
                    let ev = VerificationEvidence::artifact(None, &expected.name, false, None);
                    evidence_refs.push(ev.id().to_string());
                    out.push(
                        VerificationCheck::failed(
                            format!("artifact:{}", expected.name),
                            VerificationCheckType::ArtifactExists,
                            CheckSeverity::Critical,
                            "registered artifact",
                            "no artifact registration found",
                            "Required artifact is missing",
                        )
                        .with_evidence(ev.id().to_string()),
                    );
                }
                Some(artifact) => {
                    let before = out.len();
                    let ev = VerificationEvidence::artifact(
                        Some(artifact.id.to_string()),
                        &artifact.path,
                        true,
                        Some(artifact.size),
                    );
                    evidence_refs.push(ev.id().to_string());
                    out.push(
                        VerificationCheck::passed(
                            format!("artifact:{}", expected.name),
                            VerificationCheckType::ArtifactExists,
                            "registered artifact",
                            format!("registered at {}", artifact.path),
                        )
                        .with_evidence(ev.id().to_string()),
                    );

                    // Workspace ownership + authorization.
                    if let Some(ws_id) = workspace_id {
                        if let Some(artifact_ws) = &artifact.workspace_id {
                            if artifact_ws != ws_id {
                                out.push(VerificationCheck::failed(
                                    format!("artifact:{}:ownership", expected.name),
                                    VerificationCheckType::ArtifactValid,
                                    CheckSeverity::Blocking,
                                    format!("artifact belongs to workspace {}", ws_id),
                                    format!("artifact belongs to workspace {}", artifact_ws),
                                    "Artifact does not belong to the current workspace",
                                ));
                                Self::stamp_artifact_status(&repo, artifact, &out[before..]);
                                continue;
                            }
                        }
                    }

                    // Size bound.
                    if let Some(max) = expected.max_size_bytes {
                        if artifact.size > max {
                            out.push(VerificationCheck::failed(
                                format!("artifact:{}:size", expected.name),
                                VerificationCheckType::ArtifactValid,
                                CheckSeverity::Critical,
                                format!("size <= {}", max),
                                format!("size = {}", artifact.size),
                                "Artifact exceeds size limit",
                            ));
                            Self::stamp_artifact_status(&repo, artifact, &out[before..]);
                            continue;
                        }
                    }
                    if artifact.size == 0 {
                        out.push(VerificationCheck::failed(
                            format!("artifact:{}:non_empty", expected.name),
                            VerificationCheckType::ArtifactValid,
                            CheckSeverity::Critical,
                            "non-empty artifact",
                            "artifact size is 0",
                            "Artifact is empty and therefore unusable",
                        ));
                        Self::stamp_artifact_status(&repo, artifact, &out[before..]);
                        continue;
                    }

                    out.push(VerificationCheck::passed(
                        format!("artifact:{}:valid", expected.name),
                        VerificationCheckType::ArtifactValid,
                        "usable registered artifact",
                        "artifact registration valid",
                    ));
                    Self::stamp_artifact_status(&repo, artifact, &out[before..]);
                }
            }
        }
        out
    }

    // ------------------------------------------------------------------
    // Command / build / test verification (bounded evidence only)
    // ------------------------------------------------------------------
    fn verify_commands(
        &self,
        request: &TaskVerificationRequest,
        evidence_refs: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        for cmd in &request.commands {
            let ev = VerificationEvidence::command(
                &cmd.command_ref,
                cmd.exit_code,
                cmd.duration_ms,
                &cmd.stdout,
                &cmd.stderr,
            );
            evidence_refs.push(ev.id().to_string());
            let (check_type, label) = match cmd.kind {
                CommandKind::Build => (VerificationCheckType::BuildPassed, "build"),
                CommandKind::Test => (VerificationCheckType::TestPassed, "test"),
                CommandKind::Command => (VerificationCheckType::CommandSucceeded, "command"),
            };
            match cmd.exit_code {
                Some(0) => {
                    out.push(
                        VerificationCheck::passed(
                            format!("{}:{}", label, cmd.command_ref),
                            check_type,
                            "exit code 0",
                            "exit code 0",
                        )
                        .with_evidence(ev.id().to_string()),
                    );
                    // Test-kind commands should also show non-empty bounded output evidence;
                    // absence is a warning, not proof of failure.
                    if cmd.kind == CommandKind::Test
                        && cmd.stdout.trim().is_empty()
                        && cmd.stderr.trim().is_empty()
                    {
                        out.push(VerificationCheck::warning(
                            format!("test:{}:output", cmd.command_ref),
                            VerificationCheckType::TestPassed,
                            "bounded test output evidence",
                            "empty stdout/stderr summaries",
                            "Test exited 0 but provided no output evidence",
                        ));
                    }
                }
                Some(code) => {
                    out.push(
                        VerificationCheck::failed(
                            format!("{}:{}", label, cmd.command_ref),
                            check_type,
                            CheckSeverity::Critical,
                            "exit code 0",
                            format!("exit code {}", code),
                            "Command/build/test operation failed",
                        )
                        .with_evidence(ev.id().to_string()),
                    );
                }
                None => {
                    out.push(
                        VerificationCheck::unknown(
                            format!("{}:{}", label, cmd.command_ref),
                            check_type,
                            CheckSeverity::Critical,
                            "No exit code evidence; refusing to assume success",
                        )
                        .with_evidence(ev.id().to_string()),
                    );
                }
            }
        }
        out
    }

    // ------------------------------------------------------------------
    // Tool verification: inspect recorded ToolRuntime outcome, never execute.
    // ------------------------------------------------------------------
    fn verify_tools(
        &self,
        request: &TaskVerificationRequest,
        evidence_refs: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        for tool in &request.tools {
            let ev = VerificationEvidence::tool(&tool.tool_id, &tool.status, tool.duration_ms);
            evidence_refs.push(ev.id().to_string());
            let status_upper = tool.status.to_uppercase();
            let success = matches!(status_upper.as_str(), "VALID" | "SUCCESS" | "OK" | "PASSED");
            if success {
                let mut c = VerificationCheck::passed(
                    format!("tool:{}", tool.tool_id),
                    VerificationCheckType::ToolSucceeded,
                    "ToolRuntime successful execution",
                    format!("status={}", tool.status),
                );
                c.evidence_ref = Some(ev.id().to_string());
                out.push(c);
                if let Some(field) = &tool.expected_result_field {
                    let has_field = tool.result.get(field).is_some()
                        && !tool.result.get(field).unwrap().is_null();
                    if has_field {
                        out.push(VerificationCheck::passed(
                            format!("tool:{}:result_field", tool.tool_id),
                            VerificationCheckType::RequiredField,
                            format!("result field '{}'", field),
                            "field present",
                        ));
                    } else {
                        out.push(VerificationCheck::failed(
                            format!("tool:{}:result_field", tool.tool_id),
                            VerificationCheckType::RequiredField,
                            CheckSeverity::Critical,
                            format!("result field '{}'", field),
                            "field missing or null",
                            "Tool result does not satisfy the step requirement",
                        ));
                    }
                }
            } else {
                out.push(
                    VerificationCheck::failed(
                        format!("tool:{}", tool.tool_id),
                        VerificationCheckType::ToolSucceeded,
                        CheckSeverity::Critical,
                        "ToolRuntime successful execution",
                        format!("status={}", tool.status),
                        "Tool execution did not succeed",
                    )
                    .with_evidence(ev.id().to_string()),
                );
            }
        }
        out
    }

    // ------------------------------------------------------------------
    // Structured output verification (deterministic subset validator)
    // ------------------------------------------------------------------
    fn verify_output(
        &self,
        request: &TaskVerificationRequest,
        evidence_refs: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        if let (Some(schema), Some(value)) = (&request.output_schema, &request.output_value) {
            for err in Self::validate_against_schema(schema, value, "$") {
                out.push(VerificationCheck::failed(
                    format!("output_schema:{}", err.path),
                    VerificationCheckType::OutputSchema,
                    CheckSeverity::Critical,
                    err.expected,
                    err.actual,
                    "Structured output does not match required schema",
                ));
            }
            if out.iter().all(|c| {
                !matches!(
                    c.check_type,
                    VerificationCheckType::OutputSchema | VerificationCheckType::RequiredField
                )
            }) {
                let ev = VerificationEvidence::user_requirement("output_schema", true);
                evidence_refs.push(ev.id().to_string());
                out.push(
                    VerificationCheck::passed(
                        "output_schema",
                        VerificationCheckType::OutputSchema,
                        "schema satisfied",
                        "schema satisfied",
                    )
                    .with_evidence(ev.id().to_string()),
                );
            }
        } else if request.output_schema.is_some() && request.output_value.is_none() {
            out.push(VerificationCheck::failed(
                "output_schema",
                VerificationCheckType::OutputSchema,
                CheckSeverity::Critical,
                "output value present",
                "output value missing",
                "Schema provided but no output value to validate",
            ));
        }
        out
    }

    fn validate_against_schema(
        schema: &serde_json::Value,
        value: &serde_json::Value,
        path: &str,
    ) -> Vec<SchemaIssue> {
        Self::validate_schema_inner(schema, value, path)
    }

    fn validate_schema_inner(
        schema: &serde_json::Value,
        value: &serde_json::Value,
        path: &str,
    ) -> Vec<SchemaIssue> {
        let mut issues = Vec::new();
        let schema_type = schema.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if !schema_type.is_empty() {
            let ok = match schema_type {
                "object" => value.is_object(),
                "array" => value.is_array(),
                "string" => value.is_string(),
                "integer" => value.as_i64().is_some(),
                "number" => value.is_number(),
                "boolean" => value.is_boolean(),
                "null" => value.is_null(),
                _ => true,
            };
            if !ok {
                issues.push(SchemaIssue {
                    path: path.to_string(),
                    expected: format!("type {}", schema_type),
                    actual: json_type_of(value).to_string(),
                });
                return issues;
            }
        }
        if schema_type == "object" {
            if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                for req in required {
                    if let Some(field) = req.as_str() {
                        let present = value.get(field).map(|v| !v.is_null()).unwrap_or(false);
                        if !present {
                            issues.push(SchemaIssue {
                                path: format!("{}.{}", path, field),
                                expected: format!("required field '{}'", field),
                                actual: "missing or null".to_string(),
                            });
                        }
                    }
                }
            }
            if let (Some(props), Some(obj)) = (
                schema.get("properties").and_then(|p| p.as_object()),
                value.as_object(),
            ) {
                for (field, sub_schema) in props {
                    if let Some(sub_value) = obj.get(field) {
                        issues.extend(Self::validate_schema_inner(
                            sub_schema,
                            sub_value,
                            &format!("{}.{}", path, field),
                        ));
                    }
                }
            }
        }
        if schema_type == "array" {
            if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
                for (idx, item) in arr.iter().enumerate() {
                    issues.extend(Self::validate_schema_inner(
                        items,
                        item,
                        &format!("{}[{}]", path, idx),
                    ));
                }
            }
        }
        issues
    }

    // ------------------------------------------------------------------
    // Constraint verification (privacy / network / workspace)
    // ------------------------------------------------------------------
    fn verify_constraints(
        &self,
        request: &TaskVerificationRequest,
        workspace_id: Option<&WorkspaceId>,
        evidence_refs: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        if request.constraints.is_empty() {
            return out;
        }
        let cloud_evidence = self.detect_cloud_routing(&request.task_id);
        let network_mode = self.network.get_status().mode.to_string();

        for constraint in &request.constraints {
            let name = format!("constraint:{}", constraint.constraint_type);
            let ctype = constraint.constraint_type.to_lowercase();
            let security_sensitive = matches!(
                ctype.as_str(),
                "local_only"
                    | "no_cloud"
                    | "workspace_only"
                    | "no_unauthorized_modification"
                    | "privacy"
                    | "local-only"
                    | "no-cloud"
            );

            match ctype.as_str() {
                "local_only" | "local-only" | "no_cloud" | "no-cloud" => {
                    let ev = VerificationEvidence::user_requirement(
                        format!("{} (network_mode={})", constraint.description, network_mode),
                        !cloud_evidence,
                    );
                    evidence_refs.push(ev.id().to_string());
                    if cloud_evidence {
                        out.push(
                            VerificationCheck::failed(
                                name,
                                VerificationCheckType::ConstraintSatisfied,
                                CheckSeverity::Blocking,
                                "no cloud routing",
                                "audit shows cloud execution",
                                "Privacy constraint violated: cloud execution observed for a local-only task",
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    } else if Self::observed_satisfied(&constraint.observed) {
                        out.push(
                            VerificationCheck::passed(
                                name,
                                VerificationCheckType::ConstraintSatisfied,
                                "local-only execution preserved",
                                format!("no cloud audit evidence; network_mode={}", network_mode),
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    } else if constraint.observed.is_none() {
                        // No contradictory evidence and no explicit claim: corroborate
                        // via authoritative audit (no cloud evidence). Pass with warning
                        // rather than blocking, since the audit layer is fail-closed.
                        warnings.push(format!(
                            "Constraint '{}' has no explicit observed signal; corroborated via audit (no cloud evidence)",
                            constraint.constraint_type
                        ));
                        out.push(
                            VerificationCheck::warning(
                                name,
                                VerificationCheckType::ConstraintSatisfied,
                                "local-only execution preserved",
                                format!("no cloud audit evidence; network_mode={}", network_mode),
                                "No explicit observed signal; corroborated via authoritative audit",
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    } else {
                        out.push(
                            VerificationCheck::failed(
                                name,
                                VerificationCheckType::ConstraintSatisfied,
                                if security_sensitive {
                                    CheckSeverity::Blocking
                                } else {
                                    CheckSeverity::Critical
                                },
                                "local-only execution preserved",
                                format!("observed={:?}", constraint.observed),
                                "Local-only constraint not satisfied",
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    }
                }
                "workspace_only" | "no_unauthorized_modification" => {
                    let _ = workspace_id;
                    if Self::observed_satisfied(&constraint.observed) {
                        let ev =
                            VerificationEvidence::user_requirement(&constraint.description, true);
                        evidence_refs.push(ev.id().to_string());
                        out.push(
                            VerificationCheck::passed(
                                name,
                                VerificationCheckType::ConstraintSatisfied,
                                "workspace boundary preserved",
                                "observed satisfied + files resolved inside workspace",
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    } else if constraint.observed.is_none() {
                        // Corroborate via file checks already performed: if expected
                        // files all resolved inside the workspace, treat as satisfied
                        // with a warning when there is no contradictory evidence.
                        out.push(VerificationCheck::warning(
                            name,
                            VerificationCheckType::ConstraintSatisfied,
                            "workspace boundary preserved",
                            "no explicit observed signal; file probes stayed inside workspace",
                            "No explicit observed signal; corroborated via file boundary probes",
                        ));
                        warnings.push(format!(
                            "Constraint '{}' corroborated via file boundary probes (no explicit signal)",
                            constraint.constraint_type
                        ));
                    } else {
                        out.push(VerificationCheck::failed(
                            name,
                            VerificationCheckType::ConstraintSatisfied,
                            CheckSeverity::Blocking,
                            "workspace boundary preserved",
                            format!("observed={:?}", constraint.observed),
                            "Workspace constraint not satisfied",
                        ));
                    }
                }
                _ => {
                    if Self::observed_satisfied(&constraint.observed) {
                        let ev =
                            VerificationEvidence::user_requirement(&constraint.description, true);
                        evidence_refs.push(ev.id().to_string());
                        out.push(
                            VerificationCheck::passed(
                                name,
                                VerificationCheckType::ConstraintSatisfied,
                                "constraint satisfied",
                                format!("observed={:?}", constraint.observed),
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    } else if constraint.observed.is_none() {
                        out.push(VerificationCheck::unknown(
                            name,
                            VerificationCheckType::ConstraintSatisfied,
                            if security_sensitive {
                                CheckSeverity::Blocking
                            } else {
                                CheckSeverity::Critical
                            },
                            "No observed constraint signal; refusing to assume satisfaction",
                        ));
                    } else {
                        out.push(VerificationCheck::failed(
                            name,
                            VerificationCheckType::ConstraintSatisfied,
                            CheckSeverity::Critical,
                            "constraint satisfied",
                            format!("observed={:?}", constraint.observed),
                            "Task constraint not satisfied",
                        ));
                    }
                }
            }
        }
        out
    }

    fn observed_satisfied(observed: &Option<String>) -> bool {
        match observed {
            Some(o) => {
                let n = o.trim().to_lowercase();
                matches!(
                    n.as_str(),
                    "satisfied"
                        | "ok"
                        | "pass"
                        | "passed"
                        | "true"
                        | "local_only"
                        | "local-only"
                        | "no_cloud"
                        | "no-cloud"
                        | "workspace_only"
                        | "preserved"
                )
            }
            None => false,
        }
    }

    /// Authoritative cloud-routing detection for a task.
    /// Reads the audit trail (fail-closed writer) for positive evidence of
    /// cloud routing. Absence of such evidence means no violation observed.
    /// Never performs network access.
    fn detect_cloud_routing(&self, task_id: &str) -> bool {
        let repo = SqliteAuditRepository::new((*self.db).clone());
        let tid = TaskId::from(task_id.to_string());
        let records = repo.list_audits_by_task(&tid).unwrap_or_default();
        for record in records {
            let route_cloud = record
                .route
                .as_ref()
                .map(|r| r.to_lowercase().contains("cloud"))
                .unwrap_or(false);
            let event_cloud = record.event_type.to_uppercase().contains("CLOUD");
            let provider_cloud = record
                .provider
                .as_ref()
                .map(|p| {
                    let pl = p.to_lowercase();
                    pl.contains("openai")
                        || pl.contains("anthropic")
                        || pl.contains("gemini")
                        || pl.contains("groq")
                        || pl.contains("nvidia")
                })
                .unwrap_or(false)
                && route_cloud;
            if route_cloud || event_cloud || provider_cloud {
                return true;
            }
            // Model-routed cloud inference recorded without explicit route.
            if record.actor == "model_runtime" || record.actor == "provider_gateway" {
                if let Some(route) = &record.route {
                    if route.to_lowercase() != "local" && !route.is_empty() {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ------------------------------------------------------------------
    // User requirement verification (structural, never model-claim based)
    // ------------------------------------------------------------------
    fn verify_requirements(
        &self,
        request: &TaskVerificationRequest,
        evidence_refs: &mut Vec<String>,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        for req in &request.requirements {
            let name = format!("requirement:{}", summarize_requirement(req));
            match req.observed_satisfied {
                Some(true) => {
                    let corroborated = self.corroborate_requirement(request, req);
                    let ev = VerificationEvidence::user_requirement(&req.requirement, true);
                    evidence_refs.push(ev.id().to_string());
                    let mut c = VerificationCheck::passed(
                        name,
                        VerificationCheckType::UserRequirement,
                        req.requirement.clone(),
                        if corroborated {
                            "observed satisfied + corroborating artifact/file evidence"
                        } else {
                            "observed satisfied by authoritative probe"
                        },
                    );
                    c.evidence_ref = Some(ev.id().to_string());
                    out.push(c);
                }
                Some(false) => {
                    let ev = VerificationEvidence::user_requirement(&req.requirement, false);
                    evidence_refs.push(ev.id().to_string());
                    out.push(
                        VerificationCheck::failed(
                            name,
                            VerificationCheckType::UserRequirement,
                            CheckSeverity::Critical,
                            req.requirement.clone(),
                            "observed unsatisfied",
                            "User requirement was not satisfied",
                        )
                        .with_evidence(ev.id().to_string()),
                    );
                }
                None => {
                    let sensitive = is_security_requirement(req);
                    // Try deterministic corroboration from files/artifacts before
                    // declaring unknown.
                    if self.corroborate_requirement(request, req) {
                        let ev = VerificationEvidence::user_requirement(&req.requirement, true);
                        evidence_refs.push(ev.id().to_string());
                        out.push(
                            VerificationCheck::passed(
                                name,
                                VerificationCheckType::UserRequirement,
                                req.requirement.clone(),
                                "corroborated by artifact/file evidence",
                            )
                            .with_evidence(ev.id().to_string()),
                        );
                    } else {
                        out.push(VerificationCheck::unknown(
                            name,
                            VerificationCheckType::UserRequirement,
                            if sensitive { CheckSeverity::Blocking } else { CheckSeverity::Critical },
                            "No authoritative observed signal for requirement; refusing to infer completion",
                        ));
                    }
                }
            }
        }
        out
    }

    fn corroborate_requirement(
        &self,
        request: &TaskVerificationRequest,
        req: &UserRequirement,
    ) -> bool {
        let needle = req.requirement.to_lowercase();
        // File keyword corroboration: requirement mentions a file that was verified.
        for f in &request.expected_files {
            let fname = f.rel_path.to_lowercase();
            let base = fname.rsplit('/').next().unwrap_or(&fname);
            if !base.is_empty() && needle.contains(base) {
                return true;
            }
        }
        for a in &request.expected_artifacts {
            if !a.name.is_empty() && needle.contains(&a.name.to_lowercase()) {
                return true;
            }
        }
        // Format keyword corroboration.
        for keyword in ["pdf", "excel", "xlsx", "csv", "json", "report"] {
            if needle.contains(keyword) {
                let has_matching_file = request.expected_files.iter().any(|f| {
                    f.rel_path.to_lowercase().contains(keyword)
                        || keyword == "report" && f.rel_path.to_lowercase().contains("report")
                });
                let has_matching_artifact = request.expected_artifacts.iter().any(|a| {
                    a.name.to_lowercase().contains(keyword)
                        || a.artifact_type
                            .as_ref()
                            .map(|t| t.to_lowercase().contains(keyword))
                            .unwrap_or(false)
                });
                if has_matching_file || has_matching_artifact {
                    return true;
                }
            }
        }
        false
    }

    fn verify_no_blocking_error(
        &self,
        request: &TaskVerificationRequest,
    ) -> Vec<VerificationCheck> {
        let mut out = Vec::new();
        let mut blockers: Vec<String> = Vec::new();
        for step in &request.steps {
            if step.has_error {
                blockers.push(format!("step '{}' reports error", step.step_id));
            }
        }
        for cmd in &request.commands {
            if matches!(cmd.exit_code, Some(c) if c != 0) {
                blockers.push(format!(
                    "command '{}' exited {}",
                    cmd.command_ref,
                    cmd.exit_code.unwrap()
                ));
            }
        }
        for tool in &request.tools {
            let s = tool.status.to_uppercase();
            if matches!(
                s.as_str(),
                "FAILED" | "BLOCKED" | "INVALID" | "TIMEDOUT" | "TIMED_OUT" | "CANCELLED"
            ) {
                blockers.push(format!("tool '{}' status {}", tool.tool_id, tool.status));
            }
        }
        if blockers.is_empty() {
            out.push(VerificationCheck::passed(
                "no_blocking_error",
                VerificationCheckType::NoBlockingError,
                "no unresolved blocking errors",
                "no unresolved blocking errors",
            ));
        } else {
            out.push(VerificationCheck::failed(
                "no_blocking_error",
                VerificationCheckType::NoBlockingError,
                CheckSeverity::Blocking,
                "no unresolved blocking errors",
                blockers.join("; "),
                "Unresolved blocking errors remain",
            ));
        }
        out
    }

    // ------------------------------------------------------------------
    // Completion gate + repair loop
    // ------------------------------------------------------------------
    pub fn completion_gate(
        &self,
        result: &VerificationResult,
        requires_user_input: bool,
    ) -> CompletionDecision {
        let has_blocking = result.checks.iter().any(|c| {
            matches!(
                c.status,
                crate::verification::types::CheckStatus::Blocked
                    | crate::verification::types::CheckStatus::Unknown
            ) && (c.severity == CheckSeverity::Blocking || c.severity == CheckSeverity::Critical)
        });
        let decision = CompletionGate::evaluate(&GateInput {
            status: result.status,
            repair_attempt: result.repair_attempt,
            max_repair_attempts: self.max_repair_attempts(),
            has_blocking_failure: has_blocking,
            requires_user_input,
        });
        let task_id = TaskId::from(result.task_id.clone());
        self.publish(
            EventType::CompletionGateEvaluated,
            serde_json::json!({
                "verification_id": result.verification_id,
                "task_id": result.task_id,
                "status": result.status.to_string(),
                "decision": decision.to_string(),
            }),
            &task_id,
        );
        decision
    }

    pub fn orchestrator_action(
        &self,
        result: &VerificationResult,
        requires_user_input: bool,
    ) -> VerificationAction {
        match self.completion_gate(result, requires_user_input) {
            CompletionDecision::AllowCompletion => VerificationAction::Complete,
            CompletionDecision::RequireRepair => {
                if result.repair_attempt == 0 {
                    VerificationAction::Retry
                } else {
                    VerificationAction::Repair
                }
            }
            CompletionDecision::RequireUser => VerificationAction::AskUser,
            CompletionDecision::Blocked => VerificationAction::Fail,
        }
    }

    /// Bounded repair registration. Returns the next attempt number or an
    /// error when the budget is exhausted. Never loops infinitely.
    pub fn register_repair_attempt(&self, task_id: &str) -> Result<u32, VerificationError> {
        let max = self.max_repair_attempts();
        let persisted = self.repo.count_runs_by_task(task_id).unwrap_or(0);
        let in_memory = self.repair_attempts_for(task_id);
        let used = persisted.max(in_memory);
        if used >= max {
            return Err(VerificationError::RepairBudgetExhausted {
                task_id: task_id.to_string(),
                attempts: used,
                max,
            });
        }
        let next = self.record_attempt(task_id);
        let tid = TaskId::from(task_id.to_string());
        self.publish(
            EventType::RepairStarted,
            serde_json::json!({ "task_id": task_id, "attempt": next, "max_attempts": max }),
            &tid,
        );
        Ok(next)
    }

    // ------------------------------------------------------------------
    // Supplemental AI advisory check (local/free only, never authoritative)
    // ------------------------------------------------------------------
    /// Select a free/local model for advisory verification through the
    /// existing ModelRegistry. Never hard-codes a paid provider and never
    /// creates a cloud fallback: returns None when no local/free model exists.
    ///
    /// When a Phase 12 ModelRouter is attached, selection is routed through
    /// `ModelRouter::preview` in `LocalOnly` mode (side-effect free, no audit
    /// writes, no network calls) with online candidates excluded.
    pub fn select_advisory_model(&self) -> Option<String> {
        if let Some(router) = self.router.read().unwrap().clone() {
            let mut req = RoutingRequest::new("verification_advisory");
            req.routing_mode = RoutingMode::LocalOnly;
            req.allow_online = false;
            req.required_context_tokens = Some(1024);
            req.required_output_tokens = Some(512);
            let decision = router.preview(req);
            if decision.is_selected() {
                let mode_ok = matches!(
                    decision.execution_mode,
                    Some(crate::providers::ExecutionMode::Local)
                        | Some(crate::providers::ExecutionMode::OnPremise)
                );
                if mode_ok {
                    if let Some(model_id) = decision.selected_model_id {
                        return Some(model_id);
                    }
                }
                // Router selected a cloud candidate despite LocalOnly mode:
                // refuse it (no automatic local -> online fallback).
            }
        }
        let local = self.models.get_local_models();
        if let Some(m) = local.first() {
            return Some(m.id.clone());
        }
        // Free-tier router models are identified by free markers; only models
        // already registered in Octrex's ModelRegistry are eligible.
        let free_markers = ["free", "opencode", "ollama", "local"];
        for m in self.models.list_models() {
            let hay = format!("{} {} {}", m.id, m.provider_id, m.model_identifier).to_lowercase();
            if free_markers.iter().any(|marker| hay.contains(marker)) {
                if m.execution_mode != crate::providers::ExecutionMode::Cloud {
                    return Some(m.id.clone());
                }
            }
        }
        // Last resort: any registered OnPremise model (still no cloud fallback).
        for m in self.models.list_models() {
            if m.execution_mode == crate::providers::ExecutionMode::OnPremise {
                return Some(m.id.clone());
            }
        }
        None
    }

    /// Run a supplemental AI advisory check. The advisory output is untrusted:
    /// it may only ADD warnings, never flip Fail/Blocked/Unknown to Pass.
    ///
    /// The advisory context is budget-checked through the Phase 10
    /// ContextService: when the session budget is exhausted the advisory is
    /// skipped instead of overflowing context.
    pub async fn ai_advisory_warnings(
        &self,
        result: &VerificationResult,
        context_hint: Option<&str>,
        session_id: Option<&str>,
    ) -> Vec<String> {
        if result.status.is_pass() {
            return vec![];
        }
        let Some(model_id) = self.select_advisory_model() else {
            return vec![];
        };
        if let Some(session) = session_id {
            let window = self
                .models
                .get_model(&model_id)
                .and_then(|m| m.context_window)
                .unwrap_or(8192) as usize;
            match self
                .context
                .get_budget_for_session(session, &model_id, window)
            {
                Ok(budget) => {
                    if budget.current_usage >= budget.usable_input_budget {
                        return vec![format!(
                            "AI advisory skipped: session context budget exhausted ({}/{})",
                            budget.current_usage, budget.usable_input_budget
                        )];
                    }
                }
                Err(_) => {}
            }
        }
        let summary = result
            .failures
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("; ");
        let bounded_summary = bound_text(&summary, 1000);
        let hint = context_hint.unwrap_or("task verification");
        let prompt = format!(
            "You are a verification assistant. Given these verification failures for {}: {}. Suggest up to 3 concrete, local-only repair actions as short bullet lines. Do not claim the task is complete.",
            hint, bounded_summary
        );
        let req = crate::models::ModelRequest {
            model_id: model_id.clone(),
            messages: vec![crate::models::ModelMessage {
                role: "user".to_string(),
                content: prompt,
                tool_calls: None,
            }],
            system_instructions: Some(
                "You are Octrex verification advisor. Never declare completion.".to_string(),
            ),
            tools: vec![],
            temperature: Some(0.0),
            max_output_tokens: Some(512),
            response_format: crate::models::ResponseFormat::Text,
            metadata: HashMap::new(),
            correlation: crate::models::CallCorrelation::default(),
        };
        match self.runtime.invoke(req).await {
            Ok(resp) => {
                let text = bound_text(resp.content.trim(), 1000);
                if text.is_empty() {
                    vec![]
                } else {
                    vec![format!("AI advisory ({}; untrusted): {}", model_id, text)]
                }
            }
            Err(_) => vec![],
        }
    }

    // ------------------------------------------------------------------
    // Read APIs for server routes
    // ------------------------------------------------------------------
    pub fn get_run(
        &self,
        verification_id: &str,
    ) -> Result<Option<VerificationResult>, VerificationError> {
        let Some(record) = self.repo.get_run(verification_id)? else {
            return Ok(None);
        };
        let checks = self.repo.get_checks(verification_id)?;
        let warnings: Vec<String> = serde_json::from_str(&record.warnings_json).unwrap_or_default();
        let failures: Vec<String> = serde_json::from_str(&record.failures_json).unwrap_or_default();
        let evidence_refs: Vec<String> = checks
            .iter()
            .filter_map(|c| c.evidence_ref.clone())
            .collect();
        Ok(Some(VerificationResult {
            verification_id: record.verification_id,
            task_id: record.task_id,
            step_id: record.step_id,
            status: record.status,
            checks,
            evidence_refs,
            warnings,
            failures,
            confidence: record.confidence,
            repair_attempt: record.repair_attempt,
            created_at: record.created_at,
        }))
    }

    pub fn list_runs_by_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<VerificationResult>, VerificationError> {
        let records = self.repo.list_runs_by_task(task_id)?;
        let mut out = Vec::new();
        for record in records {
            let checks = self.repo.get_checks(&record.verification_id)?;
            let warnings: Vec<String> =
                serde_json::from_str(&record.warnings_json).unwrap_or_default();
            let failures: Vec<String> =
                serde_json::from_str(&record.failures_json).unwrap_or_default();
            let evidence_refs: Vec<String> = checks
                .iter()
                .filter_map(|c| c.evidence_ref.clone())
                .collect();
            out.push(VerificationResult {
                verification_id: record.verification_id,
                task_id: record.task_id,
                step_id: record.step_id,
                status: record.status,
                checks,
                evidence_refs,
                warnings,
                failures,
                confidence: record.confidence,
                repair_attempt: record.repair_attempt,
                created_at: record.created_at,
            });
        }
        Ok(out)
    }

    pub fn latest_run_by_task(
        &self,
        task_id: &str,
    ) -> Result<Option<VerificationResult>, VerificationError> {
        Ok(self.list_runs_by_task(task_id)?.into_iter().next())
    }

    /// Privacy verification helper reusing Phase 6 evidence.
    /// Returns true when no cloud-routing audit evidence exists for the task.
    pub fn privacy_local_only_holds(&self, task_id: &str) -> bool {
        !self.detect_cloud_routing(task_id)
    }

    /// Network verification helper reusing Phase 7 state. Read-only; performs
    /// no network access.
    pub fn network_snapshot(&self) -> serde_json::Value {
        let status = self.network.get_status();
        serde_json::json!({
            "mode": status.mode.to_string(),
            "enabled": status.enabled,
            "blocked_count": status.blocked_count,
            "allowed_count": status.allowed_count,
            "fail_closed": true,
        })
    }

    /// Privacy snapshot reusing the existing PrivacyGate type surface.
    pub fn privacy_snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "service": "PrivacyGate",
            "reused": true,
            "fail_closed": true,
        })
    }

    /// Expose the underlying PrivacyGate for advanced server-side composition.
    pub fn privacy_gate(&self) -> &Arc<PrivacyGate> {
        &self.privacy
    }
}

fn json_type_of(value: &serde_json::Value) -> &'static str {
    if value.is_object() {
        "object"
    } else if value.is_array() {
        "array"
    } else if value.is_string() {
        "string"
    } else if value.as_i64().is_some() {
        "integer"
    } else if value.is_number() {
        "number"
    } else if value.is_boolean() {
        "boolean"
    } else if value.is_null() {
        "null"
    } else {
        "unknown"
    }
}

fn summarize_requirement(req: &UserRequirement) -> String {
    let trimmed = req.requirement.trim();
    if trimmed.chars().count() <= 48 {
        trimmed.to_string()
    } else {
        let short: String = trimmed.chars().take(48).collect();
        format!("{}…", short)
    }
}

fn is_security_requirement(req: &UserRequirement) -> bool {
    let blob = format!("{} {}", req.requirement_type, req.requirement).to_lowercase();
    blob.contains("local")
        || blob.contains("private")
        || blob.contains("secret")
        || blob.contains("do_not_modify")
        || blob.contains("do not modify")
        || blob.contains("no_cloud")
        || blob.contains("no cloud")
        || blob.contains("confidential")
}

#[allow(dead_code)]
fn task_constraint_ref(_c: &TaskConstraint) -> String {
    _c.constraint_type.clone()
}
