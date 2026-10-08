# Octrex Phase 9: Model Context Protocol (MCP) Security Architecture

## Overview

Octrex provides a centralized Model Context Protocol (MCP) Security Boundary. External MCP servers are treated as untrusted third-party entities by default. MCP servers cannot self-authorize, gain unrestricted filesystem/network access, or escalate permissions.

## Key Security Architecture Components

### 1. Trust Levels
- `UNTRUSTED` (Default): Subject to strict fail-closed capability policies. Cannot execute arbitrary processes or access secrets.
- `RESTRICTED`: Scoped strictly to the active workspace directory.
- `TRUSTED`: Explicitly elevated by user consent for specified endpoints.

### 2. Tool Namespacing
All tools exposed by MCP servers are automatically namespaced using the format:
`mcp.<server_id>.<tool_name>`
This prevents tool impersonation or collisions with built-in tools.

### 3. Immediate Server Revocation
When an MCP server is disabled in `McpRegistry`:
- All active capability grants associated with the server are invalidated.
- All subsequent tool execution requests immediately fail closed with `McpDenied`.

### 4. Transport Isolation
- **stdio / Local Process**: Commands must pass through process sandbox checks.
- **HTTP / HTTPS / Remote Stream**: Egress requests must pass through Phase 7 Network Security Boundary (SSRF protection, DNS validation, domain allowlisting).

### 5. Untrusted Output Defense
All MCP server responses are wrapped in `UntrustedToolOutput`, sanitized for credentials, truncated at payload limits, and scanned for prompt injection attempts (e.g., instructions attempting to grant permissions or bypass security policies).

## Invariants Implemented

- **INVARIANT 15**: MCP servers are untrusted by default.
- **INVARIANT 16**: Disabled MCP servers cannot execute tools.
- **INVARIANT 17**: MCP output cannot alter security policy or grant capabilities.
- **INVARIANT 18**: MCP network traffic delegates to Phase 7 Network Security.
- **INVARIANT 19**: MCP filesystem access delegates to Phase 8 Filesystem Security.
- **INVARIANT 20**: All MCP connection events are logged to SQLite audit records.

## Status

- **IMPLEMENTED**: `McpRegistry`, `McpPolicyEngine`, namespaced tool IDs, trust levels, output sanitization, disable/enable lifecycle, API handlers, and Next.js settings UI (`/settings/mcp`).
- **FUTURE**: Dynamic discovery of remote MCP SSE transports with mutual TLS authentication.
