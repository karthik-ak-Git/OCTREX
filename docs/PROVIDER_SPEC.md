# OCTREX CODE V4 — Provider Infrastructure & Gateway Specification

## 1. Gateway Overview

The Universal Model Gateway decouples OCTREX from individual provider API specifications. All external AI interactions pass through provider-specific adapters enforcing a unified internal contract.

```
                              ┌───────────────────────────┐
                              │     UNIVERSAL GATEWAY     │
                              └─────────────┬─────────────┘
                                            │
               ┌────────────────────────────┼────────────────────────────┐
               ▼                            ▼                            ▼
      ┌──────────────────┐         ┌──────────────────┐         ┌──────────────────┐
      │  Gemini Adapter  │         │OpenRouter Adapter│         │  Ollama Adapter  │
      └──────────────────┘         └──────────────────┘         └──────────────────┘
               │                            │                            │
               ▼                            ▼                            ▼
      Google Gemini API              OpenRouter API              Local Ollama Server
```

---

## 2. Normalized Provider Adapter Interface

Each provider adapter implements the standard contract:

```typescript
interface ProviderAdapter {
  id: string;
  name: string;
  
  listModels(): Promise<ModelDescriptor[]>;
  healthCheck(): Promise<ProviderHealthStatus>;
  
  chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse>;
  stream(request: NormalizedChatRequest): AsyncIterable<NormalizedStreamChunk>;
  
  getCapabilities(modelId: string): ModelCapabilities;
  validateCredentials(credentials: ProviderCredentials): Promise<boolean>;
  normalizeError(error: any): NormalizedError;
}
```

---

## 3. Supported Providers & Specifications

1. **Google Gemini API**
   - Native REST / SDK integration.
   - High context window support, native structured output, multimodal capabilities.
2. **OpenRouter**
   - Unified cloud provider endpoint.
   - Multi-vendor fallback access.
3. **NVIDIA Hosted NIM**
   - NVIDIA cloud-hosted NIM microservices.
4. **Local / Custom OpenAI-Compatible Endpoints**
   - Local NIM, LM Studio, vLLM, or custom remote endpoints.
   - **ngrok Note:** ngrok tunnels are transport paths pointing to OpenAI-compatible endpoints, NOT distinct AI providers.
5. **Groq**
   - Ultra-low latency inference for quick tasks (search, classification, lightweight edits).
6. **Ollama**
   - Native local model runner for zero-cost / offline operations.

---

## 4. Health Check System

Health monitor runs background probes to evaluate provider availability without waiting for user coding tasks.

### Status Definitions
- `HEALTHY`: Full connectivity, authenticated, models accessible, streaming & tools validated.
- `DEGRADED`: High latency or elevated failure rates, but operational.
- `RATE_LIMITED`: HTTP 429 triggered; temporary backoff active.
- `AUTH_ERROR`: HTTP 401/403 invalid API key or permission denied.
- `OFFLINE`: Network unreachable, DNS failure, or server connection down.
- `UNSUPPORTED`: Endpoint active but lacks required model specs.
- `UNKNOWN`: Probe pending initial execution.

---

## 5. Model Registry & Capabilities

Every model entry tracks granular capabilities:

```typescript
interface ModelDescriptor {
  providerId: string;
  modelId: string;
  displayName: string;
  contextWindow: number;
  maxOutputTokens: number;
  supportsTools: boolean;
  supportsVision: boolean;
  supportsStreaming: boolean;
  supportsStructuredOutput: boolean;
  supportsReasoning: boolean;
  isLocal: boolean;
  isFree: boolean;
  health: ProviderHealthStatus;
}
```

---

## 6. Error Normalization

Raw vendor errors map into internal normalized error codes:

| Vendor Code | Internal Standard Code | Meaning | Router Action |
| :--- | :--- | :--- | :--- |
| `400` | `INVALID_REQUEST` | Malformed parameters / unsupported feature | Fail fast, revise payload |
| `401` / `403` | `AUTH_FAILURE` | Invalid key / forbidden access | Mark provider `AUTH_ERROR`, trip circuit |
| `404` | `MODEL_NOT_FOUND` | Specified model ID invalid or deprecated | Update registry, fallback model |
| `408` | `TIMEOUT` | Gateway or upstream timeout | Exponential backoff, fallback |
| `413` | `CONTEXT_EXCEEDED` | Request exceeds context budget | Trigger context compaction |
| `429` | `RATE_LIMITED` | Quota exceeded or rate limit | Mark `RATE_LIMITED`, switch provider |
| `500` - `504` | `PROVIDER_UNAVAILABLE` | Vendor server crash / outage | Trip Circuit Breaker, execute fallback |

---

## 7. Circuit Breaker & Fallback Architecture

- **Failure Threshold:** 3 consecutive server/network errors (5xx/429/timeout) within 60 seconds trips the Circuit Breaker to `OPEN`.
- **Cooldown Period:** Circuit remains `OPEN` for 30 seconds before transitioning to `HALF-OPEN` for probe validation.
- **Fallback Execution:** When a primary model/provider fails or trips its breaker, the Model Router seamlessly redirects active agent requests to the next qualified healthy model according to the active strategy mode.
