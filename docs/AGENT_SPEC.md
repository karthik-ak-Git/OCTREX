# OCTREX CODE V4 — Agent Architecture & Lifecycle Specification

## 1. Multi-Agent Orchestration Model

OCTREX V4 employs a multi-agent orchestration architecture. Tasks are decomposed into specialized logical roles rather than relying on a single monolithic conversational prompt.

```
                              ┌───────────────────────────┐
                              │       MANAGER AGENT       │
                              └─────────────┬─────────────┘
                                            │ Coordinates
                                            ▼
                              ┌───────────────────────────┐
                              │       PLANNER AGENT       │
                              └─────────────┬─────────────┘
                                            │ Generates Plan
                                            ▼
                              ┌───────────────────────────┐
                              │        CODER AGENT        │
                              └─────────────┬─────────────┘
                                            │ Edits Code
                                            ▼
                              ┌───────────────────────────┐
                              │       TESTER AGENT        │
                              └─────────────┬─────────────┘
                                            │ Executes Tests
                                            ▼
                    ┌───────────────────────┴───────────────────────┐
                    │ (Pass)                                 (Fail) │
                    ▼                                               ▼
      ┌───────────────────────────┐                   ┌───────────────────────────┐
      │      REVIEWER AGENT       │                   │      DEBUGGER AGENT       │
      └─────────────┬─────────────┘                   └─────────────┬─────────────┘
                    │ Verifies                                      │ Repairs Code
                    ▼                                               └───────┐
               VERIFIED RESULT                                              ▼
                                                                       CODER AGENT
```

---

## 2. Logical Specialist Roles

1. **MANAGER AGENT**
   - **Responsibility:** Interprets high-level user prompts, manages global task execution, monitors progress, updates state machine.
   - **Tools:** Session control, state transitions, sub-agent spawning.
2. **PLANNER AGENT**
   - **Responsibility:** Analyzes repository context, dependency trees, and existing patterns to formulate structured plans and acceptance criteria.
   - **Tools:** Read files, repository search, AST symbol inspection.
3. **CODER AGENT**
   - **Responsibility:** Executes concrete file edits, creates patches, updates code structure.
   - **Tools:** File write/patch, filesystem tools, Git checkpointing.
4. **DEBUGGER AGENT**
   - **Responsibility:** Analyzes build logs, test failure tracebacks, runtime exceptions; identifies root cause and crafts targeted fixes.
   - **Tools:** Log inspection, traceback parser, file read/patch.
5. **TESTER AGENT**
   - **Responsibility:** Runs target build scripts, typecheckers, unit tests, and integration test suites.
   - **Tools:** Terminal execution, test runner interface.
6. **REVIEWER AGENT**
   - **Responsibility:** Performs independent code quality, security, and acceptance criteria review on proposed changes before final verification.
   - **Tools:** Git diff inspector, code analyzer.

---

## 3. Dynamic Model Decoupling

Agent roles are **never static-bound** to specific models or vendors.
- When an agent initializes, it requests a model handle from the **Model Router** matching its required capability (e.g. `PLANNER` requests a high-reasoning model; `TESTER` requests a fast/lightweight model).
- If a model or provider fails mid-operation, the agent obtains a replacement model from the Router without losing state.

---

## 4. Task State Machine

A task transitions through explicit, observable states:

```
[RECEIVED] ──> [UNDERSTANDING] ──> [REPOSITORY_ANALYSIS] ──> [PLANNING]
                                                                  │
                                                                  ▼
[VERIFIED] <── [REVIEWING] <── [TESTING] <── [DEBUGGING] <── [IMPLEMENTING]
    │                             │
    ├── (Failed Review) ──────────┘
    │
    └── (Unrecoverable Error) ──> [FAILED]
```

---

## 5. Tournament & Consensus Mode (Multi-Solution Evaluation)

For high-complexity tasks, OCTREX V4 can launch parallel candidate paths:

1. `CODER_A` generates Solution A in Git Worktree A.
2. `CODER_B` generates Solution B in Git Worktree B.
3. `TESTER` evaluates builds and test suites independently on both worktrees.
4. `REVIEWER` inspects code diffs and performance metrics.
5. `MANAGER` selects or merges the verified winning candidate.

---

## 6. Task Cancellation & Task Lifecycle Safety

- Every task is associated with an explicit `CancellationToken`.
- User cancellation signals immediately propagate down:
  `Cancel Task -> Abort Model Requests -> Kill Running Terminal Process -> Revert Git Workspace`
- Eliminates zombie processes or background memory leaks.
