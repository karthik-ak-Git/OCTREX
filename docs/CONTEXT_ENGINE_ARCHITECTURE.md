# Octrex Context Engine Architecture (Phase 10)

## 1. Overview

The **Octrex Context Engine** is a production-grade context budgeting, assembly, compaction, and conversation state management subsystem.

It guarantees that model requests are constructed **deterministically** and **safely**, preventing naive history concatenation, token overflow, role escalation attacks, cross-workspace/session data leakage, and security policy bypasses.

---

## 2. Target Architecture Diagram

```
User Request / System Action
           |
           v
+-------------------------------------------------------+
|                 Context Engine Service                |
|                                                       |
|  1. Context Sources & Ingestion                       |
|     - System / Company / Security / Privacy Policies   |
|     - User Request & Conversation History             |
|     - Durable Task State & Verification Status        |
|     - Workspace Metadata & Selected File Contents     |
|     - Untrusted Tool Results & Model Outputs          |
|                                                       |
|  2. Classification Propagation & Trust Model          |
|     - Derived Classification = max(source_classifications)
|     - Trust Assignment by Origin Boundary             |
|                                                       |
|  3. Priority Selection (P0 - P4)                       |
|     - P0 (Mandatory Policies) NEVER evicted           |
|     - P1 (Current Task / User Request)                |
|     - P2 (Recent Conversation / Tool Results)         |
|     - P3 (Secondary Workspace Context)                |
|     - P4 (Optional Background Context)                |
|                                                       |
|  4. Token Budget Engine                               |
|     - Invariant: Input + Output Reserve (25%) +       |
|       Safety Margin (5%) + Overhead <= Window         |
|                                                       |
|  5. Bounded Compaction Engine                         |
|     - Bounded max rounds (default: 5)                 |
|     - Task-critical fact & objective preservation     |
|     - Classification preservation (no downgrade)     |
|                                                       |
|  6. Role Safety & Isolation Validation                |
|     - Data plane items CANNOT assume System role      |
|     - Session & Workspace boundary isolation          |
+-------------------------------------------------------+
           |
           v
     Assembled Context
           |
           v
      Model Runtime -> Provider Gateway -> LLM
           |
           v
     Model Output Ingestion -> Task State & Context Checkpoint
```

---

## 3. Core Security & Design Invariants

1. **Explicit Assembly Only:** Model requests are constructed by priority scoring and token budgeting, never by blind concatenation.
2. **Deterministic Token Budget Invariant:**
   $$\text{usable\_input\_budget} + \text{reserved\_output} + \text{safety\_margin} + \text{fixed\_overhead} \le \text{context\_window}$$
3. **Default Reserve:** 25% of context window is reserved for output generation by default (or explicitly configured `max_output_tokens`).
4. **Mandatory P0 Protection:** Mandatory control-plane policies (System, Company, Security, Privacy policies) have priority 0 and **cannot be evicted**. If P0 items alone exceed the usable input budget, context build fails closed with `ContextError::MandatoryContextTooLarge`.
5. **Role Safety Enforcement:** File contents, tool execution outputs, model outputs, and compaction summaries are data-plane context items. They **cannot** be assigned the `System` role. Attempting to assign `System` role to data-plane content yields `ContextError::InvalidContextRole`.
6. **Conservative Classification Propagation:** Derived context items inherit the maximum classification level of all contributing sources ($\text{classification} = \max(\text{sources})$). Compaction summaries never downgrade classification.
7. **Cross-Session & Workspace Isolation:** Context items are strictly scoped by `session_id` and `workspace_id`. Items from Session A cannot enter Session B prompt context.
8. **No Secret Data Leakage:** Diagnostic summaries, UI preview responses, and audit event logs use `safe_summary()` to redact raw content for `SECRET` or `RESTRICTED` items.
9. **No Fallback to Online Cloud:** Context Engine operates locally and deterministically. Unrecognized context decisions fail closed.

---

## 4. Bounded Compaction Subsystem

When the total token count of selected context items exceeds `usable_input_budget`:

- Compaction is triggered automatically prior to prompt dispatch.
- Compaction runs up to a configurable `max_rounds` limit (default: 5).
- Each round preserves:
  - Mandatory P0 policy items
  - Current User Request
  - Active Task State & Verification Status
- Older/secondary messages and tool outputs are summarized into a structured `CompactionSummary` `ContextItem` with role `Assistant`.
- If context still exceeds budget after `max_rounds`, compaction fails closed with `ContextError::CompactionFailed`.

---

## 5. Persistence & Database Schema (Migration v6)

- `context_items`: Per-session/task context item history with source, role, trust_level, classification, priority, token_count, and inclusion_reason.
- `context_checkpoints`: Durable checkpoints capturing selected item IDs and budget snapshots.
- `context_compactions`: Audit log of compaction events (rounds, tokens_before, tokens_after, classification).

---

## 6. Integration Boundaries

- **Phase 9 (Tool Runtime & MCP):** Tool execution outputs are ingested into Context Engine via `ContextService::ingest_tool_result()`. Context Engine consumes `ToolResponse` but does not authorize or execute tools.
- **Phase 6 (Privacy Gate):** Classification levels (`PUBLIC`, `INTERNAL`, `CONFIDENTIAL`, `RESTRICTED`, `SECRET`) and `TrustLevel` enums are reused directly from `octrex-core/src/privacy`.
- **ApplicationState & AgentEngine:** `ContextService` is registered as `context_engine_service` in `ApplicationState`. `AgentEngine::execute_with_context()` bridges agent execution with budget inspection and output ingestion.

---

## 7. Verification Results

- All 113 Rust core unit tests & DB integration tests pass: `cargo test --workspace`
- Workspace compilation check: `cargo check --workspace` (0 errors)
- Next.js 16 App Router build: `pnpm build` in `apps/web` (0 errors, static prerendering complete)
