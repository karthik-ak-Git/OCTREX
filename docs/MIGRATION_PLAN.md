# OCTREX CODE V4 — Staged Migration Plan

## 1. Migration Strategy Overview

OCTREX CODE V4 moves from the legacy monolithic implementation to a modular, decoupled architecture in 12 distinct phases. The migration enforces backward compatibility where appropriate and guarantees test verification at each phase.

---

## 2. Phase-by-Phase Roadmap

### Phase 0: Audit & Architecture Specification (COMPLETED)
- Complete workspace inspection.
- Define target specifications for Gateway, Agents, Tools, Context, Security, and Testing.
- Establish document baseline in `docs/`.

### Phase 1: Universal Gateway Core & Normalized Contracts
- Define `ProviderAdapter` typescript/python interface contracts.
- Define normalized types: `CompletionRequest`, `CompletionResponse`, `StreamEvent`, `ModelCapabilities`, `NormalizedError`.
- Build mock provider suite for deterministic testing without external API calls.

### Phase 2: Provider Adapters & Health Infrastructure
- Implement individual provider adapters:
  - Google Gemini Adapter
  - OpenRouter Adapter
  - NVIDIA Hosted & Local NIM Adapter
  - OpenAI-Compatible Adapter (Custom / ngrok-tunneled)
  - Groq Adapter
  - Ollama Adapter
- Implement `ProviderHealthMonitor` (pings, capability discovery, status classification).

### Phase 3: Model Registry & Capability Layer
- Build Model Registry for dynamic discovery and capability mapping (`context_size`, `supports_tools`, `supports_vision`, `supports_streaming`, `supports_structured_output`, `supports_reasoning`).
- Implement Error Normalization middleware (mapping 400, 401, 403, 404, 408, 413, 429, 500+ to standardized internal codes).

### Phase 4: Model Router, Fallback Engine & Circuit Breakers
- Build `ModelRouter` supporting strategy modes (`AUTO`, `FAST`, `POWERFUL`, `FREE_ONLY`, `LOCAL_ONLY`, `CUSTOM`).
- Implement per-provider Circuit Breakers and exponential backoff retry handlers.
- Dynamic fallback routing logic when primary provider triggers rate limits or outages.

### Phase 5: Repository Intelligence & Context Engine
- Implement workspace indexing, symbol extraction (AST/ripgrep), file dependency graph builder.
- Build Context Window Manager: system prompt assembly, prompt budgeting, context compaction, tool output pruning.
- Build persistent project memory store (conventions, test commands, previous decisions).

### Phase 6: Tool Engine & Safeguards
- Implement standardized tools with strict schemas: Filesystem (read/patch), Terminal (command execution with stream cancellation), Git (status, diff, checkpoint), Testing, Symbol Search.
- Implement Git workspace checkpoints and rollback handlers.
- Enforce Security Permission Layer (`LOW`, `MEDIUM`, `HIGH RISK` operational gates).

### Phase 7: Agent Orchestration Layer
- Implement `AgentOrchestrator` and Task State Machine.
- Build specialized logical agent runner classes: MANAGER, PLANNER, CODER, DEBUGGER, TESTER, REVIEWER.
- Decouple agent roles from static models; integrate dynamic binding via `ModelRouter`.

### Phase 8: Verification Engine & Repair Loop
- Implement automated test runner and build checker.
- Implement the repair loop: `PLAN -> IMPLEMENT -> RUN -> TEST -> FAIL -> REPAIR -> RETEST -> REVIEW -> VERIFY`.
- Generate structured evidence report cards.

### Phase 9: Workspace Isolation & Consensus/Tournament Mode
- Implement Git worktree isolation for concurrent agent operations.
- Build candidate solution evaluator (Tournament Mode) for competing agent branches.

### Phase 10: Frontend Integration Contracts & Event Streaming
- Define unified WebSocket / Server-Sent Events (SSE) event protocols.
- Standardize stream payloads (`task_updated`, `agent_started`, `tool_executed`, `terminal_chunk`, `diff_generated`, `provider_health_changed`).

### Phase 11: End-to-End Hardening & Automated Test Suite
- Comprehensive unit test coverage for Gateway, Router, Context, Tools, Agents.
- Integration test suite using Mock Providers.
- End-to-end task simulation and verification tests.
