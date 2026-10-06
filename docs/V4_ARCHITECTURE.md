# OCTREX CODE V4 — Target Architecture Specification

## 1. System Architecture Diagram

```
                              ┌──────────────────────────────────┐
                              │          USER EXPERIENCE         │
                              │ (Desktop / CLI / Frontend API)   │
                              └────────────────┬─────────────────┘
                                               │ Events / Streaming
                                               ▼
                              ┌──────────────────────────────────┐
                              │       TASK / SESSION ENGINE      │
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
                    │ INTELLIGENCE  │  │ (FS, Term,    │  │    ENGINE     │
                    │ & CONTEXT     │  │  Git, Tests)  │  │ (Build/Tests) │
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
                           │ Gemini │ │ Open   │ │ Local/ │ ...
                           │Adapter │ │ Router │ │ Ollama │
                           └────────┘ └────────┘ └────────┘
```

---

## 2. Core Architectural Pillars

### 2.1 Universal Model Gateway
- Normalizes request formats, tool definitions, and streaming responses across all AI vendors.
- Provider adapters for:
  1. Google Gemini API
  2. OpenRouter
  3. NVIDIA hosted NIM
  4. Local / Custom OpenAI-compatible endpoints (Ollama, LM Studio, ngrok-tunneled endpoints)
  5. Groq
  6. Ollama

### 2.2 Model Router & Circuit Breaker
- Dynamic model selection based on strategy mode (`AUTO`, `FAST`, `POWERFUL`, `FREE_ONLY`, `LOCAL_ONLY`, `CUSTOM`).
- Real-time provider health tracking (`HEALTHY`, `DEGRADED`, `RATE_LIMITED`, `AUTH_ERROR`, `OFFLINE`, `UNSUPPORTED`).
- Circuit breaker pattern to temporarily bypass failing providers (429, 500, timeouts).

### 2.3 Agent Orchestration Engine
- Decouples agent logical roles from specific AI models.
- Core Logical Roles:
  - **MANAGER:** Evaluates user intent, coordinates tasks, tracks lifecycle.
  - **PLANNER:** Analyzes repository context, builds implementation plans and acceptance criteria.
  - **CODER:** Executes file edits and code updates using precise patch tools.
  - **DEBUGGER:** Diagnoses build/test failures, analyzes tracebacks, proposes targeted fixes.
  - **TESTER:** Orchestrates unit, integration, and build checks.
  - **REVIEWER:** Performs independent verification against requirements and security/quality standards.

### 2.4 Repository Intelligence & Context Engine
- Multi-tier repo understanding: ripgrep, AST parsing, symbol search, dependency graph.
- Context window control: prompt budgeting, tool output pruning, conversation compaction, persistent project memory.

### 2.5 Tool Engine & Workspace Security
- Standardized tool contracts for Filesystem, Terminal, Git, Testing, and Symbol analysis.
- Multi-level security permission model (`LOW`, `MEDIUM`, `HIGH RISK`).
- Git-backed workspace checkpoints and rollback capabilities.

### 2.6 Verification Engine
- Replaces conversational confidence with empirical execution evidence.
- Produces verifiable report cards:
  `BUILD PASS | TESTS 100% PASS | TYPECHECK PASS | LINT PASS | REVIEW PASS -> VERIFIED`

---

## 3. Core Task Lifecycle Flow

```
RECEIVED ──> REPOSITORY_ANALYSIS ──> PLANNING ──> IMPLEMENTING
                                                       │
                                                       ▼
VERIFIED <── REVIEWING <── TESTING (PASS) <── RUN TESTS & BUILD
   │                                                   │
   │ (FAILED)                                (FAIL)    ▼
   └──────────> REPAIR LOOP <───────── DEBUGGING & REPAIR
```

---

## 4. Strategy & Routing Modes

1. **AUTO:** Dynamically picks optimal provider/model based on task difficulty, tool requirements, and current provider health.
2. **FAST:** Prioritizes low-latency models (e.g. Groq / Gemini Flash).
3. **POWERFUL:** Prioritizes high-reasoning frontier models for complex planning and refactoring.
4. **FREE ONLY:** Limits routing to free tier cloud models or local AI endpoints.
5. **LOCAL ONLY:** Binds all operations to local models (e.g. Ollama / local NIM).
6. **CUSTOM:** Manual selection of specific provider and model IDs.
