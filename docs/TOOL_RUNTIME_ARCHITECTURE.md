# Octrex Phase 9: Tool Runtime Architecture & Execution Governance

## Overview

The Octrex Tool Runtime provides a sovereign, fail-closed capability security boundary governing all tool definitions, requests, capability grants, and execution lifecycles. Models and tools do not receive unrestricted access to system resources.

## Architectural Principles

1. **Private by default. Cloud only by consent.**
2. **Least Privilege**: Tools receive only the capability declarations required for execution. Unknown tools or unknown capabilities result in an automatic `BLOCK`.
3. **No Direct Execution**: Neither model prompts nor frontend components can directly invoke tool functions without passing through `ToolRuntime` policy evaluation.
4. **Untrusted Tool Output**: Tool outputs are tagged as `UntrustedToolOutput`, sanitized for credentials, truncated at payload limits, and prevented from modifying security policies.

## Execution Pipeline (21 Steps)

1. **Receive ToolRequest**: Untrusted request containing `tool_id`, `arguments`, `requested_capabilities`, `task_id`, `session_id`, `workspace_id`.
2. **Validate Tool Identity**: Lookup typed `ToolId`.
3. **Lookup ToolDescriptor**: Retrieve tool metadata and declared capabilities from `ToolRegistry`.
4. **Validate Enabled State**: Verify tool is enabled in registry.
5. **Validate Input Schema**: Check argument size limits (max 256KB), required fields, path traversal parameters (`..` check), and command parameters.
6. **Determine Requested Capabilities**: Inspect requested capabilities against descriptor declarations.
7. **Evaluate System Policy**: Check system-wide deny rules.
8. **Evaluate Company Policy**: Check enterprise policy boundaries.
9. **Evaluate Security Policy**: Check security constraints.
10. **Evaluate Privacy Policy**: Integrate with Phase 6 Privacy Gate for data classification and privacy mode enforcement.
11. **Evaluate Filesystem Policy**: Integrate with Phase 8 Filesystem Security Boundary for workspace path containment.
12. **Evaluate Network Policy**: Integrate with Phase 7 Network Security Boundary for SSRF protection and DNS validation.
13. **Evaluate Permission Policy**: Check explicit permissions in SQLite repository.
14. **Determine Consent Requirement**: Check if high-risk actions (process execution, file deletion, confidential data access) require user confirmation.
15. **Create Scoped Capability Grant**: Issue temporary, scoped, auditable `CapabilityGrant`.
16. **Execute Tool**: Run tool within controlled `ToolExecutionContext` sandbox with timeout and cancellation token.
17. **Validate Tool Result**: Verify execution return structure.
18. **Sanitize Untrusted Output**: Redact credentials via EvidenceManager, truncate output (max 500KB), tag as `UntrustedToolOutput`, check prompt injection signals.
19. **Emit Events**: Publish lifecycle events (`ToolStarted`, `ToolCompleted`, `ToolFailed`, `ToolBlocked`) via `EventBus`.
20. **Audit Execution**: Write structured, redacted audit log entry to SQLite `audit_records`.
21. **Return ToolResponse**: Return sanitized `ToolResponse` to model or runtime.

## Invariants Implemented

- **INVARIANT 1**: Unknown tool => `BLOCK`.
- **INVARIANT 2**: Unknown capability => `BLOCK`.
- **INVARIANT 3**: Unknown policy => `BLOCK`.
- **INVARIANT 4**: Tool output cannot authorize itself or change security policy.
- **INVARIANT 5**: Tool A cannot inherit Tool B's privileges.
- **INVARIANT 6**: Capabilities are scoped by workspace ID and domain allowlists.
- **INVARIANT 7**: Tools do not receive raw `ApplicationState`.
- **INVARIANT 8**: Tool recursion depth is bounded (max depth: 5).
- **INVARIANT 9**: Tool concurrency is bounded (max concurrent: 10).
- **INVARIANT 10**: All executions are logged to SQLite audit records without logging sensitive secret payloads.

## Security Limitations & Future Sandboxing

- **IMPLEMENTED**: In-process controlled `ToolExecutionContext`, path traversal validation, input/output payload bounds, secret redaction, event bus notification, and policy hierarchy integration.
- **FUTURE**: OS-level containerization / microVM isolation (e.g., Firecracker / bubblewrap) for untrusted third-party native processes.
