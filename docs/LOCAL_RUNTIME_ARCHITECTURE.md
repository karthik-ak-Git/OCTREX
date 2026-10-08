# Octrex Local Runtime & Model Management (Phase 16)

Production-grade local model management and local inference runtime layer.

## 1. Runtime abstraction

`crates/octrex-core/src/local_runtime/` (new module, no duplication of
`ProviderRegistry`, `ModelRegistry`, `ModelRuntime`, `ModelRouter`,
`HardwareService`, `ContextEngine`, `PrivacyGate`, `NetworkSecurityService`,
`FilesystemSecurityService`, `ToolRuntime`, `VerificationEngine`,
`OrchestrationService`, skills/workflows/memory/artifact systems):

| File | Responsibility |
|---|---|
| `types.rs` | `LocalRuntimeDescriptor`, `LocalRuntimeHealth` (`Healthy/Degraded/Unavailable/Unknown`), `LocalModelState` (10 explicit states), `LocalModelRecord`, `ResourceEstimate`, `LocalCompatibilityReport`, metrics/stream/audit types |
| `errors.rs` | Fail-closed `LocalRuntimeError`, normalizes into existing `ModelError` |
| `security.rs` | Loopback-only endpoint validation, unsafe path/format rejection, download allowlist, model-metadata sanitizer (policy keys stripped) |
| `compatibility.rs` | Conservative VRAM/RAM estimation + hardware/context evaluation reusing existing engines |
| `adapter.rs` | `LocalRuntimeAdapter` trait + Ollama / llama.cpp-server / generic local OpenAI-compatible adapters + `LocalRuntimeProviderBridge` (`ModelProvider` impl) |
| `detector.rs` | Explicit endpoint probing only; no network scanning; executable discovery disabled by policy |
| `registry.rs` | `LocalRuntimeRegistry`, `LocalModelStore` (fixed transition table), `ExecutionSlots` (bounded queue) |
| `persistence.rs` | SQLite repository over migration v12 tables |
| `service.rs` | `LocalRuntimeService` facade: discovery, lifecycle, guarded inference, health, metrics, storage, downloads |
| `tests.rs` | 35 tests: runtime/model/inference/security/resource/audit + 15 adversarial cases |

Provider-neutral by construction: `ModelRouter` never imports runtime types.
It only sees normalized `ModelDescriptor`s (`ExecutionMode::Local`) plus
health/lifecycle signals via `ModelRegistry` availability.

Supported runtime types: `ollama` (native `/api/tags`, `/api/generate` with
true NDJSON streaming), `llama_cpp_server` (`/v1/models`,
`/v1/chat/completions` with true SSE streaming), `local_openai_compatible`
(same OpenAI-style surface), `other` (generic OpenAI-compatible fallback).

Supported model sources: runtime-discovered models (Ollama tags, OpenAI-style
model lists) and explicitly registered models (`gguf`, `ggml`,
`safetensors`, `ollama`, `openai-compatible`, `runtime-managed` formats).
Arbitrary native libraries are never loaded; model files are never executed
to inspect them.

## 2. Discovery

- Only explicitly supplied endpoints plus, on user action, the two well-known
  loopback candidates (`127.0.0.1:11434`, `127.0.0.1:8080`).
- Every endpoint is validated as loopback-only (IPv4/IPv6/`localhost`,
  brackets handled, credentials/query rejected, metadata endpoints rejected)
  and evaluated through `NetworkSecurityService` **before** any HTTP request.
  Loopback is local but never bypasses the boundary.
- Bounded timeouts (5s health, 10s listing), sanitized errors (no credentials
  or internal paths leak into public errors).
- Executable discovery returns a policy error: Octrex never executes
  discovered binaries. Users run the server and register the endpoint.

The pre-existing `OllamaAdapter` (config-driven single-model provider) is
untouched; Phase 16 adds per-runtime bridge providers
(`local-runtime-<id>`) fed by live adapter discovery.

## 3. Model discovery / registration / lifecycle

- Discovery populates the **existing** `ModelRegistry` (no second registry):
  `LocalModelStore` is the lifecycle authority; `sync_descriptor()` mirrors
  each record into `ModelRegistry` with `local_runtime_id` /
  `local_model_id` metadata.
- Registration validates: non-empty identifier, safe path rules, supported
  format, SHA-256 format when supplied, import path containment inside the
  managed model directory + `FilesystemSecurityService` boundary check,
  metadata sanitization (privacy/routing/permission/tool keys stripped and
  audited), duplicate rejection.
- Lifecycle: `Discovered → Registered → Available → Loading → Loaded →
  Running`, plus `Unloading/Unavailable/Failed/Disabled`, enforced by a fixed
  transition table. `Running`/`Loaded` are only reachable through
  runtime-confirmed transitions; registration alone never implies
  availability. Availability in `ModelRegistry` derives from lifecycle state.

## 4. Compatibility

Hardware: reuses Phase 5 `ModelCompatibilityEngine` unchanged
(`Compatible/CompatibleWithWarnings/Incompatible/Unknown`). `Unknown` never
auto-promotes.

Estimation (`compatibility.rs`, assumptions documented in code):
- per-quantization bytes/param table (fp32 4.0 GB/B … int4 ~0.55 GB/B,
  unknown defaults to fp16 upper bound), flat 512 MB runtime overhead,
  KV cache linear in context vs a 4k baseline;
- unknown parameter count → requirements stay `UNKNOWN` (no fake precision);
- every estimate flagged `is_estimate` with confidence/assumptions/warnings;
- declared requirements cross-checked against the estimate: claims below 50%
  of the conservative floor are overridden and flagged with
  `requirement_warning` metadata (adversarial case 5).

Context: reuses Phase 10 `TokenBudget::compute` invariant
(`input + reserved_output + safety_margin + fixed_overhead <= window`).
Unknown window ⇒ incompatible. Routable requires hardware OK **and**
context OK **and** lifecycle/health routable.

Low-resource behavior (tested with `fixture_b_4gb_nvidia`,
`fixture_a_8gb_nvidia`, `fixture_c_cpu_only`): GPU-pinned 70B-class models on
4 GB VRAM are `Incompatible`; CPU-only hosts get honest
`CompatibleWithWarnings` with offload/latency warnings.

## 5. Inference, streaming, cancellation

Flow: `ModelRouter → ModelRuntime → provider bridge → LocalRuntimeAdapter →
loopback HTTP`. Managed-model execution goes through
`LocalRuntimeService::execute` / `execute_stream` (behind `Arc<Self>` so the
forward task owns teardown):

- preflight: record exists, lifecycle + health routable, runtime routable,
  `PrivacyGate` evaluation (audit trail), explicit no-fallback assertion via
  existing `FallbackGuard`;
- resource gate: `ExecutionSlots` (default 1) rejects with guidance instead
  of auto-evicting; only Octrex-owned calls are ever cancelled; no PIDs;
- `execute`: `select!` on adapter future vs cancel channel vs reqwest timeout;
- `execute_stream`: true NDJSON/SSE streaming normalized to the existing
  `ModelStreamEvent` vocabulary (`TextDelta/Completed/Failed`); server
  exposes it over the existing SSE mechanism (`POST
  /api/local-inference/stream`) — no second SSE architecture;
- metrics per call: TTFT, total latency, approx tok/s (~4 chars/token,
  matching `DefaultTokenizer`), prompt/output tokens; diagnostics only,
  never influenced by model text; prompts never persisted;
- teardown always runs: slots released, lifecycle stepped back to `Loaded`
  (never stale `Running`), connection-level failures mark model + runtime
  `Unavailable` (fail closed, accurately representing "healthy but failing").

## 6. Resource management / queueing

Bounded execution slots; excess requests get `LocalRuntimeUnavailable` with
the occupant named. No automatic unload of user-critical work, no process
killing, no limit exceeding. `stop_runtime`/`disable_model` cancel only
Octrex-owned calls first.

## 7. Health

Per-runtime reports: reachable, model counts, latency, last success/error.
`Unknown` is never routable. Health/model signals sync into `ModelRegistry`
availability, which is the only channel that influences `ModelRouter`
scoring (existing pipeline — no new scoring path, no privacy influence).

## 8. Failure behavior (invariant)

**Local failure does not mean cloud fallback.** There is no fallback code in
the local runtime module: errors normalize to
`ModelError::{ModelUnavailable, ProviderUnavailable, Timeout, ...}` with a
`LocalRuntimeUnavailable` prefix, and `execute` responses carry
`no_cloud_fallback: true`. `FallbackGuard::assert_no_automatic_fallback`
rejects any `LocalOnly/Auto → online` transition. Tests assert failure
strings contain no cloud provider references.

## 9. Downloads / storage / security

- `model_storage_dir()`: OS data dir → `$HOME/.octrex` → temp fallback,
  `models/` subdir; never a hard-coded drive root, never a workspace.
- Downloads require `consent: true`, an allowlisted source
  (`configured-endpoint`, `local-file`, `runtime`), URL re-validation
  through `NetworkSecurityService` **including every redirect hop**,
  a 20 GB default cap, `.part` + atomic rename, installation ledger row.
  Checksums are format-validated and recorded; absence is warned, not hidden.
- Model files are untrusted: path/format/size checks, metadata sanitizer,
  no native-code loading, no policy mutation from model content.

## 10. Database (migration v12)

Forward-only addition, existing tables untouched:

- `local_runtimes` (health index)
- `local_runtime_models` (`(runtime_id, state)`, `registry_model_id` indexes)
- `model_installations` (consent recorded)
- `model_runtime_events` (redacted: ids + success + reason only)

`SqliteLocalRuntimeRepository` persists runtimes/models/installations/events
and writes redacted rows to the shared `audit_records` trail.

## 11. Server APIs

Runtimes: `GET /api/local-runtimes`, `POST /api/local-runtimes/discover`,
`POST /api/local-runtimes/register`, `GET /api/local-runtimes/:id`,
`POST /api/local-runtimes/:id/{refresh,test,start,stop}`.
Models: `GET /api/local-models`, `POST /api/local-models/discover`,
`POST /api/local-models/register`, `POST /api/local-models/download`,
`GET /api/local-models/routing/explain`, `GET /api/local-models/:id`,
`POST /api/local-models/:id/{enable,disable}`,
`GET /api/local-models/:id/compatibility`.
Inference: `POST /api/local-inference/{preview,execute,stream,cancel}`,
`GET /api/local-inference/metrics[/:call_id]`, `GET /api/local-storage`.

Also repaired (pre-existing breakage found during inspection): Phase 15
`/api/documents*` and `/api/artifacts*` routes were registered without
handlers; the canonical handlers are implemented in
`crates/octrex-server/src/main.rs` (duplicate repair attempt removed).

## 12. Frontend

- `/settings/local-runtime`: discovery/registration, per-runtime cards
  (refresh/test/start/stop/discover-models), storage dir, live activity
  (TTFT, tok/s, active calls, slot occupants).
- `/settings/local-models`: registry/discovery, explicit registration form,
  consent-gated download form, enable/disable, router explanation panel
  (selected vs rejected with hardware/context/lifecycle reasons).
- `/settings/local-models/[id]`: detail, budget-driven compatibility
  re-evaluation, guarded try-local inference.
- `LocalRuntimeCard`, `LocalModelCard`, `LocalModelCompatibilityPanel`,
  `ModelResourceEstimate` components; backend client + types extended;
  `/settings/models` links into the new pages (no disconnected UI).

## 13. Events / audit

New `EventType`s: `LocalRuntimeDiscovered/Registered/HealthChanged`,
`LocalModelDiscovered/Registered/Loaded/Unloaded`,
`LocalInferenceStarted/Completed/Failed/Cancelled`,
`LocalModelDownloadStarted/Completed/Failed`. Payloads carry ids + metrics
only — never prompts, secrets, keys, or document contents.

## 14. Tests

`local_runtime::tests` (35 tests, all passing, no network — mock adapters):
runtime discovery/validation/timeout/cancellation; model discovery,
registration, lifecycle, duplicates, metadata, VRAM/RAM/CPU-only/unknown
fixtures, context overflow; execution, streaming, cancellation, timeout
mapping, malformed responses, no-fallback strings; traversal/executable/
download/network-boundary/privacy/confidential enforcement; slot exhaustion;
audit redaction (asserts prompt text absent from storage). Adversarial cases
1–15 from the spec are each covered (redirect re-validation is
code-reviewed + unit-tested at the validation layer; live-redirect tests
would require network and are excluded by policy).

## 15. Known limitations

- No process management: Octrex tracks endpoint lifecycle state; starting
  `ollama serve` / `llama-server` binaries remains the user's job.
- Checksum recording without in-build cryptographic verification (warned,
  verify out-of-band).
- Tokenizer is the existing char-based estimator, not model-native.
- Single default execution slot (configurable via `LocalRuntimeConfig`).
- Downloads support direct + redirected http(s) only; no torrent/IPFS/registry protocols.
