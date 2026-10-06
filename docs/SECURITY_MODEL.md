# OCTREX CODE V4 — Security Model & Permission Specification

## 1. Overview

Autonomous code modification and terminal execution require a zero-trust, defense-in-depth architecture. OCTREX V4 enforces clear permission tiers, credential isolation, and command sanitization.

---

## 2. Risk Tier Classification

```
┌─────────────────────────────────────────────────────────────┐
│                       LOW RISK                              │
│ Read files, search project, view diffs, run safe tests      │
├─────────────────────────────────────────────────────────────┤
│                      MEDIUM RISK                            │
│ Edit source files, create branches, install dependencies     │
├─────────────────────────────────────────────────────────────┤
│                      HIGH RISK                              │
│ Delete directories, run system commands, access external net│
└─────────────────────────────────────────────────────────────┘
```

| Tier | Actions Included | Security Policy |
| :--- | :--- | :--- |
| **LOW RISK** | `file_read`, `repo_search`, `git_diff`, `test_run` | Allowed automatically |
| **MEDIUM RISK** | `file_patch`, `file_write`, `git_checkpoint`, `npm install` | Allowed with local workspace sandbox verification |
| **HIGH RISK** | `file_delete`, destructive terminal commands (`rm -rf`, system format), external HTTP requests | Requires explicit user confirmation / security policy gate |

---

## 3. Secret Management & Key Isolation

1. **Backend-Only Storage:** API keys (`GEMINI_API_KEY`, `OPENROUTER_API_KEY`, `NVIDIA_API_KEY`, `GROQ_API_KEY`) reside exclusively in backend environment variables or secure native OS keychains.
2. **Zero Frontend Exposure:** API keys are never returned in REST responses, WebSocket payloads, or exposed to browser contexts.
3. **Prompt Sanitization:** Key scanners strip sensitive tokens (`sk-...`, `AIza...`, `gsk_...`) before sending text prompts to external AI models.
4. **Log Redaction:** Terminal and application loggers automatically sanitize authorization headers and environment secrets.

---

## 4. Command Execution Safeguards

- Command blacklist prevents execution of destructive system calls (e.g. `rm -rf /`, `mkfs`, system shutdown).
- Shell calls execute within constrained working directories (`d:\OCTREX` workspace boundaries).
- Timeouts enforce auto-termination for runaway or non-terminating processes.
