# Octrex Verification & Reliability Architecture (Phase 13)

## 1. Purpose

The Verification & Reliability Engine prevents the agent from declaring a task
successful merely because a model said "Done." Completion is established only
from **structured, authoritative evidence**:

- required steps actually completed (orchestrator step store + persisted `task_steps`)
- required outputs exist (workspace-bound file probes, artifact registry)
- files are valid (non-empty, expected type/content, modification bounds)
- tools returned successful results (`ToolRuntime` recorded outcomes, never re-executed)
- commands / builds / tests exited 0 (bounded exit-code + output summaries)
- structured outputs match schema (deterministic subset validator)
- task constraints satisfied (privacy / network / workspace, checked independently)
- no unresolved blocking errors remain

A model completion claim is recorded for audit but **never trusted as evidence**.
A request carrying only a model claim yields `Blocked` (`model_claim_without_evidence`)
with confidence `0.0`.

## 2. Architecture

```
User Request
    |
    v
Phase 11 Agent Orchestrator (OrchestratorCoordinator / TaskExecutor)
    |
    v
Model / Tool / Filesystem Execution
    |
    v
VerificationEngine (crates/octrex-core/src/verification/)
    |
    +--> Step verification (plan steps + persisted task_steps)
    +--> File verification (Phase 8 FilesystemSecurityService)
    +--> Artifact verification (artifact repository + workspace ownership)
    +--> Command / build / test verification (bounded evidence)
    +--> Tool verification (Phase 9 ToolRuntime outcomes, inspect-only)
    +--> Structured output verification (deterministic schema subset)
    +--> Constraint verification (Phase 6 privacy audit + Phase 7 network state)
    +--> Requirement verification (structural user requirements)
    +--> Blocking-error sweep
    |
    v
VerificationResult { PASS | PASS_WITH_WARNINGS | FAIL | BLOCKED | UNKNOWN }
    |
    v
CompletionGate::evaluate -> AllowCompletion | RequireRepair | RequireUser | Blocked
    |
    v
Orchestrator action: Complete | Retry | Repair | Replan | AskUser | Fail
```

## 3. Modules (`crates/octrex-core/src/verification/`)

| File | Responsibility |
|---|---|
| `mod.rs` | Module surface, re-exports |
| `types.rs` | `VerificationStatus`, `CheckStatus`, `CheckSeverity`, `VerificationCheckType` (closed set), `VerificationCheck`, `VerificationResult`, `CompletionDecision`, `VerificationAction` |
| `checks.rs` | `TaskVerificationRequest` + declarative expectations (`ExpectedFile`, `ExpectedArtifact`, `CommandEvidenceInput`, `ToolEvidenceInput`, `StepEvidenceInput`, `TaskConstraint`, `UserRequirement`); request validation rejects absolute paths and `..` |
| `evidence.rs` | Provenance-carrying evidence enum; bounded text (`MAX_COMMAND_OUTPUT_CHARS = 2000`, `MAX_FILE_PREVIEW_CHARS = 2000`); never stores secrets or full contents |
| `policy.rs` | `VerificationPolicy::aggregate` (fail-closed aggregation) + evidence-only confidence |
| `gate.rs` | `CompletionGate::evaluate` |
| `repository.rs` | `SqliteVerificationRepository` over existing SQLite (`verification_runs`, `verification_checks`, migration v9) |
| `service.rs` | `VerificationEngine`: all check families, gate, bounded repair, AI advisory |
| `errors.rs` | `VerificationError` |
| `tests.rs` | Adversarial + positive tests |

The legacy `orchestration::VerificationService` trait + `StandardVerificationService`
remain only as a step-level gate inside `TaskExecutor`. Its bare-path artifact
probe is fail-closed (absolute / `..` paths are rejected because no workspace
authorization can be established), and its docs state it is **not** completion
evidence. Terminal completion is decided by the engine's `CompletionGate`.

## 4. Status model and fail-closed rules

- `Pass` — every check passed.
- `PassWithWarnings` — passed with non-blocking warnings (e.g. skipped steps,
  corroborated-but-unsignalled boundary constraints).
- `Fail` — a required check failed; repair may be possible.
- `Blocked` — a `Blocking`/`Critical` failure, or `Unknown` on a
  security-sensitive check. Never auto-retried blindly.
- `Unknown` — indeterminate. **Unknown MUST NOT become Pass.** The policy maps
  `Unknown` on security-sensitive checks (`FileExists`, `FileContent`,
  `ArtifactExists`, `ArtifactValid`, `ConstraintSatisfied`, `NoBlockingError`,
  `ToolSucceeded`, …) to `Blocked`.

Confidence (`0.0–1.0`) reflects authoritative evidence only; model claims never
raise it.

## 5. Evidence model

Evidence variants: `File`, `Tool`, `Command`, `Artifact`, `Test`, `TaskState`,
`UserRequirement`. Each carries an `evidence_id`, provenance fields, and a
timestamp. Bounded summaries only. No raw secrets, no full file dumps.

## 6. Completion gate

`CompletionGate::evaluate(GateInput { status, repair_attempt, max_repair_attempts, has_blocking_failure, requires_user_input })`:

- `Pass` / `PassWithWarnings` → `AllowCompletion`
- `Fail` + repairable + attempts left → `RequireRepair`
- `Fail` + blocking failure → `Blocked`
- `Fail` + budget exhausted / user input needed → `RequireUser`
- `Blocked` / `Unknown` → `Blocked` (fail-closed)

`VerificationEngine::orchestrator_action` maps the gate to
`Complete | Retry | Repair | Replan | AskUser | Fail`.

## 7. Check families

### 7.1 Step verification
Step outcomes come from the orchestrator's step store (`StepEvidenceInput`) plus
corroboration against persisted `task_steps` when rows exist. `COMPLETED`
without error passes; `SKIPPED` (orchestrator-waived) is a warning;
anything else fails. Dependencies pass only when every supplied step is
complete or waived.

### 7.2 File verification (Phase 8 reuse)
Every probe goes through `FilesystemSecurityService::evaluate_operation`
(`Stat` for existence/authorization, `Read` via `read_file` for content).
Workspace boundary, symlink/reparse protection, protected paths, and
classification are enforced by Phase 8 — verification cannot bypass them and
cannot become an unauthorized file-access mechanism. `workspace_id` is
required; without it the check is `Unknown` → `Blocked`.

### 7.3 Artifact verification
`ExpectedArtifact`s are matched against `SqliteArtifactRepository` rows for the
task: registration must exist, workspace ownership must match the request
workspace, size must respect `max_size_bytes`, and empty artifacts are
rejected as unusable.

### 7.4 Command / build / test verification
The engine never executes commands. It inspects `CommandEvidenceInput`
(exit code, duration, bounded stdout/stderr). Exit `0` passes; non-zero fails;
missing exit code is `Unknown` (fail-closed). Empty test output with exit 0 is
a warning.

### 7.5 Tool verification (Phase 9 reuse)
Recorded `ToolRuntime` outcomes are inspected (`VALID`/`SUCCESS`/`OK` →
pass; `FAILED`/`BLOCKED`/… → fail), including expected result fields. Tools
are never executed from verification, and capabilities can never be granted
here.

### 7.6 Structured output verification
Deterministic subset validator for `{"type": "object"|"array"|…, "required":
[…], "properties": {…}}` with basic type checks. No model judgment involved.

### 7.7 Constraint verification (Phases 6 + 7 reuse)
- `local_only` / `no_cloud`: fails `Blocking` when the task audit trail shows
  cloud routing (`detect_cloud_routing` over `audit_records`); otherwise an
  explicit `observed` satisfaction signal passes, and corroboration via audit
  (no cloud evidence) passes with a warning. No second privacy system is
  created; no network access is performed.
- `workspace_only` / `no_unauthorized_modification`: explicit signal passes;
  otherwise corroborated via in-workspace file probes with a warning.
- `output_format` / custom constraints: require an explicit satisfaction
  signal; security-sensitive customs without signals are `Unknown` → `Blocked`.

### 7.8 Requirement verification
`UserRequirement { requirement, requirement_type, observed_satisfied }`:
`Some(true)` passes (with corroboration note when file/artifact keywords
match); `Some(false)` fails; `None` attempts deterministic corroboration
(file/artifact/format keyword match) and otherwise is `Unknown` (fail-closed
for security-sensitive requirements). Model claims are never consulted.

### 7.9 Blocking-error sweep
Any step error flag, non-zero command exit, or failed/blocked tool status
fails `no_blocking_error` with `Blocking` severity.

## 8. Repair loop (Phase 11 integration)

`TaskExecutor` accepts an optional engine (`with_verification_engine`;
`OrchestratorCoordinator` / `OrchestrationService` forward it;
`ApplicationState` attaches it by default). After legacy `verify_completion`
passes, `gated_complete` builds a `TaskVerificationRequest` from the plan
(step outcomes, `WriteFile` `rel_path` probes, plan constraints) and routes the
terminal decision through the gate:

- `AllowCompletion` → `Complete`
- `RequireRepair` → `register_repair_attempt` (bounded by `max_repair_attempts`,
  default 3, clamp 1–10) → `Retry`, or `AskUser` when the budget is exhausted
- `RequireUser` → `AskUser`
- `Blocked` → `Block` (+ `TaskStatus::Blocked`)

Infinite repair loops are impossible: the budget is enforced in-memory and
cross-checked against persisted run counts.

## 9. AI verifier (supplemental, local/free only)

`select_advisory_model` routes through the Phase 12 `ModelRouter::preview` in
`LocalOnly` mode (side-effect free) with online candidates excluded; a cloud
selection is refused (no automatic local → online fallback). Fallbacks consult
only already-registered local/free models, then on-premise models. With no
eligible model, advisory is skipped — verification never depends on it.

`ai_advisory_warnings` may only **add warnings** to failed/blocked runs; it can
never flip `Fail`/`Blocked`/`Unknown` to `Pass`. Verifier output is untrusted.

No paid provider is hard-coded; no cloud fallback exists anywhere in this path.

## 10. Privacy, network, context integration

- **Privacy (Phase 6):** local-only preservation is established from the
  fail-closed audit trail; `privacy_gate` is reused (attached, inspectable via
  `privacy_snapshot`), never duplicated.
- **Network (Phase 7):** read-only `network_snapshot` (mode, counters);
  verification performs zero network access and creates no second monitor.
- **Context (Phase 10):** verification builds no separate context engine; the
  advisory prompt is bounded (1000 chars) and secret-free by construction.
- **Filesystem (Phase 8), Tools (Phase 9), Router (Phase 12):** reused as above.

## 11. Database

Migration **v9** adds (no FKs, mirroring `audit_records` style so registry-only
task IDs persist):

- `verification_runs(id, task_id, step_id, workspace_id, session_id, status, confidence, repair_attempt, warnings_json, failures_json, created_at)` + `idx_ver_runs_task`
- `verification_checks(id, verification_id, name, check_type, status, severity, expected, actual, evidence_ref, message)` + `idx_ver_checks_run`

Existing `tasks`, `task_steps`, `artifacts`, `events`, `audit_records` are
reused. No second database.

## 12. Events and audit

New `EventType`s: `VerificationCheckCompleted`, `VerificationBlocked`,
`RepairStarted`, `CompletionGateEvaluated`, `TaskVerificationAdvisory`
(plus pre-existing `VerificationStarted/Passed/Failed`). Every run publishes
per-check events and records an `audit_records` row (`actor =
"verification_engine"`, evidence references only).

## 13. API

- `GET /api/tasks/:id/verification` — run history
- `POST /api/tasks/:id/verify` — run verification (path task ID authoritative;
  body mismatch rejected fail-closed); returns result + gate + action
- `GET /api/tasks/:id/verification/:verification_id` — run detail (task
  ownership enforced)
- `POST /api/tasks/:id/verification/:verification_id/retry` — bounded repair
  registration (budget exhaustion is an error, never a loop)
- `GET /api/tasks/:id/completion` — latest verdict + gate + action;
  `NOT_VERIFIED`/`BLOCKED` when no run exists

## 14. Frontend

- Route `app/tasks/[id]/verification` (page + `TaskVerificationClient`) with
  run timeline, check detail, gate status, evidence references, retry control.
- Components: `VerificationStatus`, `VerificationChecklist`,
  `VerificationEvidence`, `CompletionGateStatus`, `RepairAttemptTimeline`,
  `VerificationWarning`, `VerificationFailure`.
- Main workspace page (`app/page.tsx`): a Verification card in the Activity tab
  bound to the latest executed task, showing verified state, gate decision,
  failure count, and a deep link. No secrets are rendered.

## 15. Security invariants

1. Model claims are never completion evidence.
2. Unknown security-sensitive state → Block/Fail (fail-closed).
3. File verification cannot bypass Phase 8.
4. Tools are never executed and capabilities never granted by verification.
5. No network access is performed for verification.
6. Repair is bounded; exhaustion escalates to the user.
7. AI advisory is warnings-only and local/free-routed; it cannot pass anything.
8. No paid model, no cloud fallback, no second privacy/network/filesystem/tool/
   context/model/agent system.

## 16. Known limitations

- The lightweight `/api/agent/execute` path marks its registry task Complete
  directly from the provider response; it does not route through the
  CompletionGate (it carries no structured completion evidence). Independently
  verified completion is established via orchestrated tasks (Phase 11, gated by
  default in `ApplicationState`) and the `/verify` + `/completion` APIs, which
  honestly report `NOT_VERIFIED` for such tasks.
- Custom constraints and natural-language requirements without explicit
  observed signals (or corroborating file/artifact keywords) resolve to
  `Unknown` → `Blocked`; orchestrators should attach deterministic probes.
- Cloud-routing detection relies on the audit trail's `route`/`provider`
  fields; a silent side-channel outside audited actors is out of scope (defense
  in depth remains with Phases 6/7 enforcement).
- The schema validator covers a pragmatic object/array subset, not full JSON
  Schema.
- `verification_runs` rows are append-only history; no retention pruning yet.
