# OCTREX Rust Core Architecture & Foundation (Phase 1)

## Overview

This document describes the Phase 1 foundational architecture of the Octrex backend application runtime built in Rust (`octrex-core` and `octrex-server`).

Rust is the authoritative runtime for the Octrex desktop suite. The Next.js frontend client communicates with Rust over a strongly typed IPC boundary (HTTP REST and Server-Sent Events stream for realtime telemetry).

```
+-------------------------------------------------------------+
|                     Next.js 16.3 UI Client                   |
+-------------------------------------------------------------+
                               |
                   Typed IPC & SSE Event Stream
                               v
+-------------------------------------------------------------+
|                  Axum Desktop Server Bridge                 |
|                      (octrex-server)                        |
+-------------------------------------------------------------+
                               |
                         ApplicationState
                               v
+-------------------------------------------------------------+
|                    Octrex Rust Core Engine                  |
|                        (octrex-core)                        |
|                                                             |
|  +-------------------+  +--------------------------------+  |
|  | ApplicationState  |  | ServiceRegistry                |  |
|  +-------------------+  +--------------------------------+  |
|  | EventBus          |  | TaskRegistry & SessionRegistry |  |
|  +-------------------+  +--------------------------------+  |
|  | ConfigLoader      |  | WorkspaceRegistry & Manager    |  |
|  +-------------------+  +--------------------------------+  |
|  | Error Model       |  | ProviderGateway & AgentEngine  |  |
|  +-------------------+  +--------------------------------+  |
+-------------------------------------------------------------+
```

---

## Foundational Subsystems

### 1. Strongly Typed Identifiers (`octrex_core::ids`)

All domain entities use strongly typed, prefixed identifiers backed by UUID v4:

- `RequestId`: `req-<uuid>`
- `TaskId`: `task-<uuid>`
- `SessionId`: `session-<uuid>`
- `WorkspaceId`: `ws-<uuid>`
- `EventId`: `evt-<uuid>`
- `ToolCallId`: `tool-<uuid>`
- `ModelCallId`: `model-<uuid>`
- `ArtifactId`: `art-<uuid>`

### 2. Application State & Lifecycle (`octrex_core::app`)

- **State Container**: `ApplicationState` holds safe, concurrent references (`Arc<RwLock<...>>` / `Arc<...>`) to configuration, registries, event bus, and gateways.
- **Lifecycle Machine**: `LifecycleManager` enforces deterministic states (`INITIALIZING` -> `READY` -> `SHUTTING_DOWN` -> `STOPPED`).

### 3. Event Bus & Envelope (`octrex_core::events`)

- **In-process Async Bus**: Powered by Tokio broadcast channels (`EventBus`).
- **Telemetry Envelope**: Every event publishes an `EventEnvelope` containing `event_id`, `event_type`, `timestamp`, optional correlation IDs (`request_id`, `task_id`, `session_id`, `workspace_id`), and typed payload JSON.
- **SSE Stream**: `octrex-server` exposes `/api/events`, allowing Next.js to stream backend activity in realtime.

### 4. Structured Error Model (`octrex_core::error`)

- Machine-readable error codes (`ErrorCode` enum): `VALIDATION_ERROR`, `NOT_FOUND`, `PERMISSION_DENIED`, `POLICY_DENIED`, `PRIVACY_BLOCKED`, `NETWORK_BLOCKED`, `PROVIDER_ERROR`, etc.
- Safe client serialization via `AppErrorResponse` (no raw stack traces or internal secrets sent to UI).

### 5. Service Registry (`octrex_core::services`)

- Lightweight dependency container (`ServiceRegistry`) tracking service statuses (`Starting`, `Ready`, `Degraded`, `Failed`).
- Core services registered at startup:
  - `event_bus_service`
  - `workspace_service`
  - `session_service`
  - `task_service`
  - `provider_service`
  - `agent_engine_service`

### 6. Domain Registries

- **`TaskRegistry`**: Thread-safe task state tracking (`Created`, `Running`, `Paused`, `Cancelled`, `Completed`, `Failed`).
- **`SessionRegistry`**: Identity tracking for active chat and execution sessions.
- **`WorkspaceRegistry`**: File tree inspection, read/write boundaries, and workspace identity.

---

## IPC Contract & API Endpoints

| Method | Endpoint | Description | Response Contract |
| --- | --- | --- | --- |
| GET | `/api/info` | Application and protocol version info | `ApplicationInfo` |
| GET | `/api/status` | Realtime runtime state and active entity counts | `RuntimeStatus` |
| GET | `/api/health` | Deep backend health check across registered services | `BackendHealth` |
| GET | `/api/config/summary` | Non-sensitive configuration summary | `ConfigurationSummary` |
| GET | `/api/events` | SSE Stream of `EventEnvelope` items | `EventSource` stream |
| GET | `/api/providers` | Live health checks for LLM providers | Provider health list |
| POST | `/api/providers/:id/connect` | Connect or update provider API key | Connection status |
| POST | `/api/workspace/inspect` | Inspect disk workspace path | `WorkspaceInfo` |
| POST | `/api/workspace/tree` | List workspace directory entries | `Vec<FileEntry>` |
| POST | `/api/workspace/read_file` | Safely read workspace file | File contents |
| POST | `/api/workspace/write_file` | Safely write/update workspace file | Bytes written |
| POST | `/api/agent/execute` | Execute agent prompt against provider | `AgentExecutionResponse` |

---

## Frontend Integration (`apps/web/lib/backend`)

The Next.js client connects using clean abstractions:
- `lib/backend/types.ts`: TypeScript contracts matching Rust IPC types.
- `lib/backend/client.ts`: `OctrexBackendClient` singleton for REST calls.
- `lib/backend/events.ts`: `OctrexEventSubscriber` singleton for SSE event streams.

---

## Verification & Testing

Run backend tests:
```bash
cargo test --workspace
```

Run frontend build & typecheck:
```bash
pnpm --filter web run build
```
