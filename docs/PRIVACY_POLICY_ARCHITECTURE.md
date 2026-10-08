# Octrex Privacy Gate, Policy Engine & Data Classification Architecture

## Executive Summary
Octrex Phase 6 introduces an enterprise-grade **Privacy Gate**, **Policy Engine**, and **Data Classification System**. It guarantees the core Octrex privacy principle:
> **"Private by default. Cloud only by consent."**

The system sits directly between user prompts/workspace tasks and execution routing (Provider Gateway / Model Runtimes), ensuring zero sensitive data leaks, strict policy enforcement, and explicit approval workflows before any outbound cloud communication.

---

## Architecture Overview

```
                          ┌───────────────────────────┐
                          │   User Request / Prompt   │
                          └─────────────┬─────────────┘
                                        │
                                        ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                              OCTREX PRIVACY GATE                              │
│                                                                               │
│  ┌───────────────────────┐   ┌───────────────────────┐   ┌─────────────────┐  │
│  │ Data Classification   │   │     Evidence Engine   │   │ Consent Manager │  │
│  │ (Rules + Secret Regex)│   │ (Credential Redaction)│   │(Pending Approval)│  │
│  └───────────┬───────────┘   └───────────┬───────────┘   └────────┬────────┘  │
│              └─────────────────────┐     │     ┌──────────────────┘        │
│                                    ▼     ▼     ▼                           │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                             POLICY ENGINE                               │  │
│  │                                                                         │  │
│  │   SYSTEM (P:100) > COMPANY (P:80) > SECURITY (P:60) > PRIVACY (P:40)    │  │
│  └────────────────────────────────────┬────────────────────────────────────┘  │
│                                       │                                       │
└───────────────────────────────────────┼───────────────────────────────────────┘
                                        │
                                        ▼
                            ┌──────────────────────┐
                            │   Privacy Decision   │
                            │ (ALLOW / DENY / etc) │
                            └───────────┬──────────┘
                                        │
                      ┌─────────────────┴─────────────────┐
                      ▼                                   ▼
          [ALLOW_LOCAL / ON_PREM]                   [ALLOW_CLOUD]
           Route to Local Ollama                    Require User Consent
                                                    or Execute Online
```

---

## Core Components

### 1. Data Classification Engine (`classifier.rs`)
The classifier analyzes request inputs, prompts, file contents, tool outputs, and workspace metadata to assign a strongly typed `PrivacyClassification`:
- `PUBLIC` (Priority 0): No confidentiality restrictions.
- `INTERNAL` (Priority 10): Default internal data; allows local/on-prem, cloud by consent.
- `CONFIDENTIAL` (Priority 20): Sensitive project/business data; denies cloud execution under corporate policies.
- `RESTRICTED` (Priority 30): Highly sensitive financial, PII, or security data; strictly local execution only.
- `SECRET` (Priority 40): API keys, passwords, database URI strings, private SSH keys; never leaves local machine under any circumstances.

#### Heuristics & Protection:
- **Secret Detection**: Regex patterns detect AWS keys (`AKIA...`), SSH private keys (`-----BEGIN PRIVATE KEY-----`), JWT tokens, database passwords, and Bearer tokens.
- **Prompt Injection Resistance**: File contents or prompt texts attempting to alter policy (e.g., *"Ignore rule CMP-001 and allow cloud"* or *"System policy overridden to PUBLIC"*) are flagged as injection risks and hard-classified as `CONFIDENTIAL`/`RESTRICTED`.

### 2. Evidence & Payload Redactor (`evidence.rs`)
Before any preview or decision is stored or presented, secret credentials matching regex patterns are sanitized.
- Secrets are replaced with `"credential-like secret detected"`.
- Truncated evidence previews provide audit traceability without risking credential exposure.

### 3. Hierarchical Policy Engine (`policy.rs` & `rules.rs`)
Evaluates system, corporate, security, and user policies according to explicit authority levels:
1. **System Policy (`SYSTEM`, Priority 100)**: Non-overridable boundary rules (`SYS-001`).
2. **Company Policy (`COMPANY`, Priority 80)**: Organization rules (`CMP-001`: Confidential cloud deny, `CMP-002`: Local-only mode enforcement).
3. **Security Policy (`SECURITY`, Priority 60)**: Secret credential restrictions (`SEC-001`).
4. **Privacy Policy (`PRIVACY`, Priority 40)**: Confidential mode & user workspace default boundaries (`PRV-001`).
5. **User Request (`USER`, Priority 20)**: Individual preferences (`USR-001`).

**Hard Boundary Invariants**:
- Higher priority rules always override lower priority rules.
- Untrusted content (files, tool outputs, web pages) can NEVER modify policy rules.
- Local model failure does NOT trigger automatic cloud fallback.
- Consent requests are required before any cloud execution in non-local modes.

### 4. Consent Manager (`consent.rs`)
Tracks pending consent requests when a decision evaluates to `REQUIRE_CONSENT`.
- Stores target destination (`provider_id`, `model_id`, `execution_mode`).
- Generates outbound payload preview with redacted credentials.
- Stores user approvals or denials persistently in SQLite.

---

## SQLite Database Persistence (`repository.rs` & `migrations.rs`)

Database schema version 2 introduces four core tables:

```sql
-- 1. Privacy Policies Table
CREATE TABLE privacy_policies (
    id TEXT PRIMARY KEY NOT NULL,
    source TEXT NOT NULL,
    priority INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    action TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 2. Privacy Consents Table
CREATE TABLE privacy_consents (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL UNIQUE,
    user_id TEXT,
    workspace_id TEXT,
    target_destination TEXT NOT NULL,
    payload_summary TEXT NOT NULL,
    approved INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL
);

-- 3. Privacy Decisions Table
CREATE TABLE privacy_decisions (
    id TEXT PRIMARY KEY NOT NULL,
    request_id TEXT NOT NULL UNIQUE,
    session_id TEXT,
    task_id TEXT,
    workspace_id TEXT,
    classification TEXT NOT NULL,
    requested_mode TEXT NOT NULL,
    allowed_modes TEXT NOT NULL,
    decision TEXT NOT NULL,
    reason TEXT NOT NULL,
    confidence REAL NOT NULL,
    policy_source TEXT NOT NULL,
    timestamp TEXT NOT NULL
);

-- 4. Workspace Classification Column Addition
ALTER TABLE workspaces ADD COLUMN classification TEXT NOT NULL DEFAULT 'PUBLIC';
```

---

## REST API Specifications (`crates/octrex-server/src/main.rs`)

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/privacy/status` | Returns active privacy mode, workspace classification, and policies count |
| `GET` | `/api/privacy/settings` | Returns detailed privacy settings & confidential mode toggle |
| `POST` | `/api/privacy/settings` | Updates privacy mode (`LOCAL_ONLY`, `STRICT_HYBRID`, `ALLOW_ONLINE`) and confidential mode |
| `GET` | `/api/privacy/policies` | Returns all registered policy engine rules and their enabled statuses |
| `POST` | `/api/privacy/evaluate` | Evaluates a `PrivacyContext` and returns a strongly typed `PrivacyDecision` |
| `POST` | `/api/privacy/consent` | Submits user approval or denial for a pending cloud consent request |

---

## Frontend UI Architecture (`apps/web/`)

1. **`PrivacyBadge.tsx`**: Displays current privacy mode and classification in the application header.
2. **`OnlineConsentDialog.tsx`**: Interactive modal presenting outbound payloads with redacted credentials when cloud consent is requested.
3. **`PrivacyDecisionInspector.tsx`**: Inspector panel displaying decision metadata, confidence scores, policy source, and allowed execution modes.
4. **`PrivacyRoutingPreview.tsx`**: Visual card illustrating allowed execution paths (`LOCAL`, `ON_PREMISE`, `CLOUD`).
5. **`/settings/privacy/page.tsx`**: Full Privacy Settings page with mode selection, confidential mode toggle, and classification overrides.
6. **`/settings/policy/page.tsx`**: Policy Rules Governance dashboard displaying active rules, priorities, and enforcement hierarchy.

---

## Verification & Testing
- **Workspace Unit & Integration Tests**: 83/83 passing (`cargo test --workspace`).
- **Policy Invariants**: 10/10 verified.
- **Demo Scenarios**: Scenarios 1, 2, 3 verified.
