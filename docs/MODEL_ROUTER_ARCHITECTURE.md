# Phase 12 Architecture: Production Model Router

## 1. Architectural Overview & Boundaries

The **Model Router** (`crates/octrex-core/src/router/`) is the production authorization-aware model selection engine for Octrex. It sits between the Task Orchestrator (Phase 11) and the Model Runtime / Provider Gateway (Phase 4), answering the critical question:

> *"Given this task, context, policy, hardware, capabilities, latency/cost requirements, and available models, which execution target is permitted and appropriate?"*

### Explicit Non-Responsibilities & Invariants
1. **The Router DOES NOT execute tools**: Tool permissioning and execution remain owned by the Tool Runtime (`crates/octrex-core/src/tools/`).
2. **The Router DOES NOT authorize network access**: External API traffic remains governed by Network Security (`crates/octrex-core/src/network/`).
3. **The Router DOES NOT replace the Privacy Gate**: Classification enforcement remains owned by `PrivacyGate` (`crates/octrex-core/src/privacy/`).
4. **The Router DOES NOT execute model calls**: Direct API invocations and streaming remain owned by `ModelRuntime` (`crates/octrex-core/src/providers/`).
5. **The Router DOES NOT perform automatic Local -> Cloud fallback**: Local execution failure or latency never triggers automatic cloud authorization.

---

## 2. Policy Priority Hierarchy

When evaluating candidate models against incoming task requirements, the router enforces a strict policy priority hierarchy:

```
[SYSTEM POLICY]
       ↓
[COMPANY POLICY]
       ↓
[SECURITY POLICY]
       ↓
[PRIVACY CLASSIFICATION]
       ↓
[PERMISSION GRANT]
       ↓
[TASK CONSTRAINTS]
       ↓
[USER PREFERENCE]
       ↓
[MODEL PREFERENCE / SCORING]
```

Higher tiers unconditionally override lower tiers. For example, if a user requests `Online` execution mode or an explicit online model, but Company Policy or Privacy Classification restricts the workspace to `LocalOnly`, the router will return a `PolicyDenied` or `Blocked` decision rather than routing online.

---

## 3. Evaluation Pipeline & Filtering

The Model Router processes requests through a deterministic multi-stage filtering and scoring pipeline (`CandidateFilter` & `CandidateScorer`):

```
Candidate Models (from ModelRegistry)
  │
  ├─► 1. Health Gate (ProviderRegistry health & error rate check)
  │
  ├─► 2. Mode Gate (Match routing mode: Auto, LocalOnly, OnPremOnly, OnlineOnly, Explicit)
  │
  ├─► 3. Privacy & Policy Gate (Consult PrivacyGate & Company Policy engine)
  │
  ├─► 4. Hardware Gate (Evaluate VRAM, RAM, CPU cores via HardwareService profile)
  │
  ├─► 5. Context Gate (Check max_input_tokens + required_output_tokens + 10% safety margin)
  │
  ├─► 6. Capability Gate (Verify chat, code, vision, tool calling requirements)
  │
  └─► Scorer Pipeline
        │
        ├─ Local Preference Weight
        ├─ Latency Requirement Match
        ├─ Capability Surplus Score
        └─ Provider Health Multiplier
        │
        ▼
   Routing Decision (Selected / Blocked / NoCompatibleModel / RequireUserSelection / etc.)
```

---

## 4. Decision Taxonomy & Evidence

Every evaluation yields a structured `RoutingDecision` (`crates/octrex-core/src/router/decision.rs`):

- `Selected`: An authorized, compatible model was identified and scored highest.
- `RequireUserSelection`: Multiple candidate models match equally or user intervention is mandated by policy.
- `Blocked`: Explicit security, company policy, or privacy classification violation.
- `NoCompatibleModel`: No registered model satisfies capability, context, or hardware requirements.
- `NoAuthorizedModel`: Models exist, but none are permitted for the workspace privacy classification.
- `ContextTooLarge`: The required prompt + safety margin exceeds all candidate model context windows.
- `HardwareIncompatible`: Local/on-prem hardware constraints (e.g. VRAM) fail candidate requirements.
- `ProviderUnavailable`: All target providers are unhealthy or unreachable.
- `PolicyDenied`: Explicit workspace/company security policy restriction.
- `Unknown`: Unrecognized error or unresolvable security constraint (fails closed).

### Security Evidence
All decisions retain diagnostic evidence without ever exposing provider credentials or secret API keys:
- `policy_evidence`: Policy rules evaluated and decision details.
- `hardware_compatibility`: Hardware checks, detected VRAM, and compatibility status.
- `context_compatibility`: Context budget breakdown and margin validation.

---

## 5. No Automatic Cloud Fallback Invariant

Octrex strictly enforces **zero automatic local-to-cloud fallback**:
- If a local model execution fails, stalls, or encounters runtime errors, the router **never** automatically resends the request to an online LLM provider.
- If a local context window is exceeded, the router **never** silently redirects to a cloud endpoint.
- Online execution requires explicit user authorization and must pass all Privacy Gate and Network Security checks independently.

---

## 6. API Endpoints & Server Integration

The server (`crates/octrex-server/src/main.rs`) exposes administrative and execution routing endpoints:

- `GET /api/router/status`: Returns router status, registered models/providers, and hardware confidence.
- `POST /api/router/evaluate`: Evaluates a full `RoutingRequest` and records decision audit logs.
- `POST /api/router/preview`: Diagnostic dry-run preview for UI inspection.
- `GET /api/router/decisions`: Returns audit log history of past routing decisions.
- `GET /api/router/decisions/:id`: Fetches detailed evidence for a specific decision.
- `GET /api/models/compatible`: Returns currently compatible model descriptors.
- `GET /api/models/recommended`: Returns hardware-optimized model recommendations.

---

## 7. Audit Tracing & Persistence

All decisions are written to the SQLite database via Migration 7 (`routing_decisions` table):
- Columns: `id`, `task_id`, `session_id`, `workspace_id`, `state`, `model_id`, `provider_id`, `execution_mode`, `reason`, `policy_evidence`, `hardware_compatibility`, `context_compatibility`, `confidence`, `created_at`.
- Events are broadcast via `EventBus`: `RoutingStarted`, `RoutingCandidatesEvaluated`, `RoutingModelSelected`, `RoutingBlocked`, `RoutingNoCompatibleModel`, `RoutingRequireUserSelection`.
