# OCTREX — PROVIDER & MODEL ABSTRACTION ARCHITECTURE (PHASE 4)

## Overview

Phase 4 establishes the production-grade **Provider + Model Abstraction Layer** for Octrex on top of the Rust application foundation (`crates/octrex-core`).

The core architectural principle of Octrex is:
> **Provider != Model**  
> **Private by Default. Cloud only by Consent.**

A **Provider** represents a service/runtime boundary (e.g. Local Ollama, NVIDIA NIM, Groq, Google Gemini, OpenCode).  
A **Model** represents a concrete inference capability exposed through a specific provider (e.g. `llama-3.3-70b-instruct`, `gemini-1.5-pro`, `opencode-free-router`).

---

## High-Level Architecture

```
                       USER REQUEST
                            │
                            ▼
                     Privacy / Policy  (Future Phase)
                            │
                            ▼
                    Model Requirements (Phase 4 Contracts)
                            │
                            ▼
                 Hardware Compatibility (Phase 5)
                            │
                            ▼
                      Model Router     (Future Phase)
                            │
                            ▼
                      Model Runtime    (Phase 4)
                            │
                            ▼
                    Provider Adapter   (Phase 4)
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
          LOCAL         ON-PREMISE       ONLINE
```

Phase 4 implements the bottom runtime layer without implementing premature fallback or automatic cloud escalation logic.

---

## Key Domain Concepts & Modules

### 1. Provider Domain (`crates/octrex-core/src/providers/`)

- `ProviderDescriptor`: Metadata describing a provider (ID, display name, provider type, execution mode, status, base URL). **Never contains API keys or secret credentials.**
- `ProviderStatus`: State of the provider (`Available`, `Unavailable`, `Degraded`, `Unknown`, `Disabled`).
- `ExecutionMode`: Semantic execution classification (`Local`, `OnPremise`, `Cloud`/`Online`).
- `ProviderRegistry`: Central dependency-injected registry managing active provider adapters.
- `HealthChecker`: Performs low-latency non-costly connectivity, authentication, and model availability checks.
- `ModelProvider` (Adapter Trait): Async Rust trait implemented by provider adapters (`OpenAICompatibleAdapter`, `GoogleAdapter`, `OllamaAdapter`, `MockAdapter`).

### 2. Model Domain (`crates/octrex-core/src/models/`)

- `ModelDescriptor`: Authoritative model descriptor (ID, provider ID, model identifier, display name, execution mode, capabilities, context window, token limits, tokenizer info, hardware requirements, availability).
- `ModelCapability`: Explicit capability flags (`TextGeneration`, `Vision`, `ImageInput`, `ImageOutput`, `ToolCalling`, `FunctionCalling`, `StructuredOutput`, `JsonOutput`, `Streaming`, `Embedding`, `CodeGeneration`).
- `ModelRequirement`: Criteria used to query matching models based on capabilities, execution mode, minimum context window, etc.
- `ModelAvailability`: Explicit model status (`Available`, `Unavailable`, `Unknown`, `Disabled`).
- `ModelRegistry`: In-memory registry enabling query and filtering of models by provider, execution mode, capabilities, and requirement rules.
- `ModelRuntime`: Core execution boundary executing normalized `ModelRequest` objects against target provider adapters and returning normalized `ModelResponse` or `ModelStreamEvent` streams.

---

## Security & Credential Isolation

1. **Credential Boundaries**: API keys and secrets reside in application configuration (`AppConfig` / `ProviderConfig`) or secure key storage and are **never** serialized into public provider/model metadata DTOs.
2. **Secret Redaction**: Errors, logs, SSE events, and public REST responses enforce strict credential masking (`[REDACTED]`).
3. **No Automatic Cloud Fallback**: If a `LOCAL` model fails, the provider abstraction **never** silently falls back to an `ONLINE` cloud model. Failures are returned explicitly to protect user privacy.

---

## REST API Integration (`crates/octrex-server`)

- `GET /api/providers`: Lists provider descriptors and health checks.
- `GET /api/providers/:id`: Retrieves specific provider descriptor and capabilities.
- `GET /api/providers/:id/models`: Lists models for a given provider.
- `GET /api/models`: Queries models filtered by `execution_mode` or `capability`.
- `GET /api/models/*id`: Retrieves detailed model descriptor by ID.
- `POST /api/providers/:id/connect`: Sets API key and refreshes provider health.

---

## Frontend Contract Alignment (`apps/web`)

TypeScript definitions in `apps/web/lib/backend/types.ts` mirror the Rust backend DTOs:
- `ExecutionMode`
- `ModelCapability`
- `ModelAvailability`
- `ProviderDescriptor`
- `ModelDescriptor`
- `ModelRequirement`
- `ProviderHealthCheck`
