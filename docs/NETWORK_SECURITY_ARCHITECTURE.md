# Octrex Network Security Architecture & Outbound Traffic Enforcement

## Executive Summary

Octrex follows the fundamental product principle:

> **"Private by default. Cloud only by consent."**

Network security in Octrex is a mandatory **SECURITY ENFORCEMENT boundary**. The architecture operates under a zero-trust model where model outputs, documents, prompt-injected instructions, web content, MCP tool results, or terminal output can **NEVER** elevate network privileges or override security policies.

---

## 1. Threat Model

Octrex defends against the following threat vectors:

1. **Prompt Injection / In-Band Priv-Esc**: Malicious documents or injected model responses instructing the agent to exfiltrate files to an external URL.
2. **Server-Side Request Forgery (SSRF)**: Requests targeting internal network ranges (`10.0.0.0/8`, `192.168.0.0/16`, `172.16.0.0/12`), loopback (`127.0.0.0/8`, `::1`), link-local addresses (`169.254.0.0/16`), or cloud metadata services (`169.254.169.254`, `metadata.google.internal`).
3. **DNS Rebinding Attacks**: Hostnames resolving to public IPs during evaluation but rebinding to local/private addresses during connection.
4. **Unsafe HTTP Redirect Hops**: Allowed initial URLs redirecting to blocked/untrusted external destinations.
5. **Silent Fallback Leaks**: Automatic fallback from blocked local execution to cloud providers.
6. **Frontend State Tampering**: Malicious frontend state attempting to authorize outbound connections without backend policy validation.

---

## 2. Network Security Boundary

The canonical Rust module `crates/octrex-core/src/network/` serves as the sole authoritative enforcement boundary for all outbound requests.

```
User Request / Model Request / Tool Invocation
                      |
                      v
      Privacy Gate (Phase 6 Integration)
                      |
                      | PrivacyDecision
                      v
          Network Policy Evaluator
                      |
         +------------+------------+
         |                         |
     [  BLOCK  ]               [ ALLOW ]
         |                         |
         v                         v
  Audit & Event          Authorized Transport
```

---

## 3. Policy Hierarchy

Policy evaluation enforces strict priority ordering. Lower-priority policies can **NEVER** override a higher-priority DENY rule.

```
1. SYSTEM POLICY     (Priority 1 - Hardcoded Invariants)
       >
2. COMPANY POLICY    (Priority 2 - Organization Rules)
       >
3. SECURITY POLICY   (Priority 3 - SSRF & IP Boundary Rules)
       >
4. PRIVACY POLICY    (Priority 4 - Data Classification & Privacy Gate)
       >
5. PERMISSION POLICY (Priority 5 - Task & Scope Authorization)
       >
6. USER POLICY       (Priority 6 - User Allowlist / Preferences)
```

Example: If Company Policy blocks external network access, a User Allowlist rule for `api.openai.com` evaluates to **BLOCK**.

---

## 4. Fail-Closed Behavior

Network enforcement adheres strictly to **Fail-Closed** invariants:

```
Unknown Network Decision ==> BLOCK
Network Service Unavailable ==> BLOCK
DNS Resolution Error ==> BLOCK
Redirect to Unapproved Domain ==> BLOCK
Missing Policy / Policy Evaluation Error ==> BLOCK
```

Octrex NEVER evaluates `unknown -> allow` or automatically retries blocked local execution through cloud providers.

---

## 5. SSRF & IP Address Protections

All outbound request IP addresses are classified prior to connection:

- **Loopback**: `127.0.0.0/8`, `::1`
- **Private IPv4**: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `100.64.0.0/10` (CGNAT)
- **Link-Local**: `169.254.0.0/16`, `fe80::/10`
- **Cloud Metadata Services**: `169.254.169.254`, `metadata.google.internal`, `100.100.100.200`
- **Multicast / Reserved / Unspecified**: `224.0.0.0/4`, `ff00::/8`, `0.0.0.0`, `255.255.255.255`

If an external request resolves to any non-public classification, it is immediately **BLOCKED**.

---

## 6. DNS Security & Rebinding Protection

Hostnames are resolved using standard socket address resolution and every resolved IP is validated against SSRF classification rules. Resolved IP addresses are attached to the normalized `NetworkEndpoint` struct so transport layers connect strictly to validated destinations.

---

## 7. Redirect Validation

Redirects do **NOT** inherit the permission of the original URL. Every redirect hop is evaluated independently against the full policy engine. If any hop is unapproved or resolves to a blocked address, the entire request chain fails with `NetworkError::RedirectBlocked`.

---

## 8. Outbound Payload Boundary & Privacy Integration

Phase 7 exposes `OutboundPayloadBoundary` to combine Phase 6 Privacy Gate decisions with Network Security enforcement:

| Privacy Gate | Network Policy | Final Disposition |
| :--- | :--- | :--- |
| ALLOW | ALLOW | **ALLOW** |
| BLOCK | ALLOW | **BLOCK** |
| ALLOW | BLOCK | **BLOCK** |
| REQUIRE_CONSENT | ALLOW | **REQUIRE_CONSENT** |

---

## 9. Operating Modes

1. **Local Only (Default)**: External network requests are strictly blocked. Loopback endpoints (e.g., local Ollama on `http://localhost:11434`) are permitted if compliant with local security policy.
2. **Restricted**: Outbound network requests allowed ONLY to host patterns present in the approved domain allowlist.
3. **Online Allowed**: Outbound requests permitted subject to policy rules, SSRF validation, DNS checks, and explicit consent. Unknown destinations remain blocked.
4. **Disabled (Kill Switch)**: All network access disabled completely.

---

## 10. Audit Behavior & Data Protection

Every network decision is logged to the embedded SQLite `audit_records` table and emitted over `EventBus`.

- **Redacted Information**: Authorization headers, cookies, API keys, password query parameters, and raw request bodies are NEVER written to logs or audit records.
- **Recorded Metadata**: Timestamp, Request ID, Source, Capability, Destination, Disposition, Reason, Policy Source, Matched Rule ID, Task ID, Workspace ID, Provider ID.

---

## 11. Implemented vs. Planned Boundaries

### IMPLEMENTED (Phase 7)
- Application-level Network Security Boundary in Rust Core (`octrex-core`).
- Fail-closed policy evaluator and policy hierarchy.
- SSRF protections (Private IP, Loopback, Link-Local, Cloud Metadata).
- Safe DNS resolver & redirect hop evaluator.
- Typed capabilities & allowlist management.
- Axum REST endpoints (`/api/network/*`).
- TypeScript client & Next.js frontend UI (`NetworkSecurityModal`, `NetworkSecurityBadge`, settings, activity inspector, dry-run tester).
- SQLite audit persistence & EventBus integration.

### PLANNED (Future Phases)
- OS-level firewall rules integration (e.g. Windows Filtering Platform / `pf` / `nftables` sandbox process binding).
- Terminal command network mediation via container/sandbox namespace isolation.
- Full MCP tool network declaration runtime validator.

---

## 12. Security Invariants Verification Summary

1. Unknown network decision -> **BLOCK** (Verified)
2. Model output cannot authorize network access -> **BLOCK** (Verified)
3. File content cannot authorize network access -> **BLOCK** (Verified)
4. Tool output cannot authorize network access -> **BLOCK** (Verified)
5. Frontend state cannot authorize network access -> **BLOCK** (Verified)
6. Lower-priority policy cannot override higher-priority DENY -> **BLOCK** (Verified)
7. Local Only mode cannot send data to external services -> **BLOCK** (Verified)
8. Blocked provider cannot be replaced automatically with online fallback -> **BLOCK** (Verified)
9. Redirect destinations evaluated independently -> **BLOCK** (Verified)
10. Private/loopback/link-local destinations blocked -> **BLOCK** (Verified)
11. Secrets never logged to audit database -> **REDACTED** (Verified)
