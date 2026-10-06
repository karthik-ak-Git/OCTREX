# CODEX HANDOFF SPECIFICATION: OCTREX CODE V4

## 1. System Overview & Final Backend Architecture

OCTREX CODE V4 is an autonomous software-engineering operating system where multiple specialist AI agents collaborate to understand repositories, plan work, modify code, execute real tools, test implementations, repair failures, review diffs, and verify results against empirical evidence.

```
                              ┌──────────────────────────────────┐
                              │     CODEX FRONTEND / PRODUCT     │
                              │ (UI, Chat, Diffs, Terminal, UX)  │
                              └────────────────┬─────────────────┘
                                               │ Structured Events / Commands
                                               ▼
                              ┌──────────────────────────────────┐
                              │         TASK ENGINE & STORE      │
                              │      (task_xxx State Machine)    │
                              └────────────────┬─────────────────┘
                                               │
                                               ▼
                              ┌──────────────────────────────────┐
                              │        AGENT ORCHESTRATOR        │
                              │  (Manager, Planner, Coder, etc.) │
                              └─┬──────────────┬──────────────┬──┘
                                │              │              │
                                ▼              ▼              ▼
                    ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
                    │  REPOSITORY   │  │  TOOL ENGINE  │  │ VERIFICATION  │
                    │ INTELLIGENCE  │  │ & PERMISSIONS │  │    ENGINE     │
                    │ & CONTEXT     │  │ (FS, Git, Dev)│  │ (Build/Tests) │
                    └───────────────┘  └───────────────┘  └───────────────┘
                                               │
                                               ▼
                              ┌──────────────────────────────────┐
                              │           MODEL ROUTER           │
                              │ (Circuit Breaker & Fallbacks)    │
                              └────────────────┬─────────────────┘
                                               │
                                               ▼
                              ┌──────────────────────────────────┐
                              │     UNIVERSAL MODEL GATEWAY      │
                              └─┬──────────┬──────────┬──────────┘
                                │          │          │
                                ▼          ▼          ▼
                           ┌────────┐ ┌────────┐ ┌────────┐
                           │ Gemini │ │ Open   │ │ NVIDIA │ ...
                           │Adapter │ │ Router │ │ Adapter│
                           └────────┘ └────────┘ └────────┘
```

---

## 2. Directory & Module Structure

```
d:\OCTREX\
├── docs/
│   ├── CURRENT_ARCHITECTURE.md
│   ├── V4_ARCHITECTURE.md
│   ├── MIGRATION_PLAN.md
│   ├── PROVIDER_SPEC.md
│   ├── AGENT_SPEC.md
│   ├── TOOL_SYSTEM.md
│   ├── CONTEXT_ENGINE.md
│   ├── SECURITY_MODEL.md
│   ├── TEST_STRATEGY.md
│   └── CODEX_HANDOFF.md
└── src/
    ├── core/
    │   ├── types/                    # Unified TypeScript type definitions
    │   ├── gateway/                  # Universal Model Gateway & Adapters
    │   │   ├── adapters/             # Gemini, OpenRouter, NVIDIA, Groq, Custom OpenAI
    │   │   ├── errorNormalizer.ts    # Normalized error mapping
    │   │   ├── streamingToolParser.ts# Fragmented JSON tool call streaming aggregator
    │   │   ├── universalGateway.ts   # Main model gateway
    │   │   ├── mockAdapter.ts        # Offline testing mock adapter
    │   │   ├── ollamaAdapter.ts      # Lazy Ollama startup adapter
    │   │   └── opencodeAdapter.ts    # OpenCode & Free model pool adapter
    │   ├── router/                   # Strategy modes (AUTO, FAST, POWERFUL), Circuit Breakers
    │   ├── context/                  # RepoIndexer, Symbol Search & ContextEngine
    │   ├── tools/                    # Filesystem, Terminal, Git & DevTools
    │   ├── security/                 # PermissionModel (17 Risk Rules), CheckpointGuard
    │   ├── task/                     # TaskStore (task_xxx IDs & lifecycle)
    │   ├── agents/                   # AgentOrchestrator & Specialist Roles
    │   ├── verification/             # VerificationEngine & AutoRepairLoop
    │   ├── memory/                   # ProjectMemoryStore (.octrex/memory.json)
    │   ├── recovery/                 # SessionRecoveryManager
    │   ├── orchestration/            # TournamentEngine (Consensus Mode)
    │   ├── cancellation/             # CancellationToken & Provider Disconnect Manager
    │   ├── events/                   # UIEventEmitter (Real-time schema events)
    │   └── contracts/                # Frozen Backend-Frontend Schemas
    ├── tests/                        # Full Automated Test Suite (49 passing tests)
    └── index.ts                      # Core Entry Point
```

---

## 3. How to Build & Run Tests

```bash
# Typecheck
npm run typecheck

# Production Build
npm run build

# Run Full Test Suite
npm test
```

---

## 4. Frontend / Backend Integration Contracts

All events emitted to the UI use the standard schema:

```typescript
export interface UIEventPayload {
  taskId: string;
  eventType: UIEventType;
  timestamp: string;
  data: Record<string, any>;
}
```

### Supported Real-time Event Types:
- `task_updated`: Emitted on state changes (`RECEIVED`, `PLANNING`, `IMPLEMENTING`, `TESTING`, `DEBUGGING`, `REVIEWING`, `VERIFIED`, `FAILED`, `CANCELLED`).
- `agent_started` / `agent_completed` / `agent_failed`: Emitted per agent lifecycle stage.
- `tool_executed`: Emitted on file edits, terminal commands, and Git operations.
- `terminal_chunk`: Real-time stdout/stderr streaming chunks.
- `diff_generated`: Unified diff updates.
- `provider_health_changed`: Real-time provider health transitions (`HEALTHY`, `DEGRADED`, `RATE_LIMITED`, `AUTH_ERROR`, `OFFLINE`).
- `model_fallback_occurred`: Emitted on dynamic fallback routing (`model.selected`, `fallback.started`, `fallback.completed`).
- `verification_completed`: Delivers the structured verification report card.

---

## 5. Security & Permission Contract

- API keys reside strictly backend-side or in secure local storage; never returned over renderer APIs.
- The 17 risk rules enforce:
  - `LOW`: Auto-permitted (reading, tests, inspection).
  - `MEDIUM`: Auto-approved by workspace policy with pre-modification Git checkpoints.
  - `HIGH` / `BLOCKED`: Blocked dangerous system deletions or piped remote scripts.

---

## 6. Files Codex May Freely Modify vs Core Contracts

### Files Codex May Create/Modify:
- Frontend UI components, views, and styling (`renderer/`, `src/ui/`, `src/components/`).
- Electron desktop window wrapper, menus, and visual panels.
- Visual diff viewers, terminal console tabs, provider settings modals, and dashboards.

### Core Contracts Codex Should NOT Rewrite:
- `src/core/types/` — Shared type schemas.
- `src/core/gateway/` — Universal Gateway, ErrorNormalizer, and streaming tool accumulator.
- `src/core/router/` — Strategy modes and CircuitBreaker logic.
- `src/core/security/` — PermissionModel and CheckpointGuard.
- `src/core/verification/` — VerificationEngine empirical validation checks.

---

# CODEX + GPT-6 ASTRA — YOUR WORK STARTS HERE

The core autonomous software engineering backend, Universal Model Gateway, Specialist Agent Orchestrator, Repository Intelligence Engine, and Verification Engine are fully built, hardened, and verified with 100% test pass rates.

### Remaining Frontend / Product Scope for Codex:
1. **Premium UI/UX Design:** Modern dark-mode IDE desktop layout (Electron / Web).
2. **Chat & Task Flow View:** Interactive prompt input, mode selector (`AUTO`, `FAST`, `POWERFUL`, `FREE_ONLY`, `LOCAL_ONLY`, `CUSTOM`), and agent step visualization.
3. **Real-time Terminal & Diff Viewer:** Split-pane visual diff viewer for modified files and live terminal log streamer.
4. **Provider Health & Model Settings Panel:** Interactive configuration UI for entering Gemini, OpenRouter, NVIDIA, Groq, Ollama, and OpenAI endpoints with live health status badges.
5. **Verification Report Card Display:** Visual badges for Build, Test Suite, Typecheck, Lint, and Independent Reviewer acceptance.
6. **Task Checkpoint & Rollback UX:** One-click rollback buttons to undo unverified or unwanted changes.
