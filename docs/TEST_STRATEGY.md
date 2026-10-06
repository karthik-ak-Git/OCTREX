# OCTREX CODE V4 — Test Strategy & Empirical Verification Specification

## 1. Multi-Tier Testing Pyramid

```
                                ▲
                               │  END-TO-END (E2E) TESTS
                              │   Autonomous Coding Flows
                             ├───┬───────────────────────┐
                            │    │  INTEGRATION TESTS    │
                           │     │  Adapters, Tools,     │
                          │      │  Routing & Fallbacks  │
                         ├───┬───┴───────────────────────┤
                        │    │    UNIT TESTS             │
                       │     │    Normalizers, Contracts,│
                      │      │    Parsers & State Engine │
                     └───────┴───────────────────────────┘
```

---

## 2. Test Execution Tiers

### 2.1 Unit Testing
- Gateway payload normalizers (OpenAI, Gemini, OpenRouter formats).
- Error normalization transformer (400, 401, 404, 413, 429, 500 mapping).
- Context truncation and log compaction logic.
- Agent task state machine transitions.

### 2.2 Integration Testing
- Provider adapter streaming parsers.
- Circuit breaker state transitions (`CLOSED` -> `OPEN` -> `HALF-OPEN`).
- Fallback route selection when primary provider returns mock 429 / 500 errors.
- Filesystem patch engine verification (exact line chunk replacements).

### 2.3 Mock / Fake Provider Infrastructure
To ensure deterministic testing without consuming real API quota or relying on network availability:
- Implementation of `MockProviderAdapter` capable of injecting canned responses, streamed chunks, HTTP error codes (400, 401, 404, 413, 429, 500), latency delays, and connection dropouts.

### 2.4 End-to-End Verification Testing
- Synthetic task flow:
  1. Create coding task.
  2. Parse requirement.
  3. Execute patch.
  4. Run build & test runner.
  5. Inject mock test failure -> trigger Debugger -> repair code -> re-test -> verify pass.
  6. Confirm final verification report generation.

---

## 3. Evidence-Based Verification Engine

OCTREX V4 requires concrete evidence before marking a task as `VERIFIED`:

```typescript
interface VerificationReport {
  taskId: string;
  timestamp: string;
  status: 'VERIFIED' | 'FAILED';
  checks: {
    buildPassed: boolean;
    testsPassed: number;
    testsTotal: number;
    typecheckPassed: boolean;
    lintPassed: boolean;
    reviewerApproved: boolean;
    acceptanceCriteriaMet: boolean;
  };
  evidenceLogs: {
    buildOutput: string;
    testSummary: string;
    reviewerComments: string;
  };
}
```
Conversational text declarations ("I fixed the issue") are rejected by the Verification Engine unless backed by a passing `VerificationReport`.
