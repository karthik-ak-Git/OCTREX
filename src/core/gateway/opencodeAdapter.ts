import {
  ProviderAdapter,
  ModelDescriptor,
  ProviderHealthStatus,
  NormalizedChatRequest,
  NormalizedChatResponse,
  StreamEvent,
  ModelCapabilities,
  ProviderCredentials,
  NormalizedError,
} from '../types/gateway.js';
import { ErrorNormalizer } from './errorNormalizer.js';

export interface OpenCodeAdapterOptions {
  baseUrl?: string;
  apiKey?: string;
  fetchFn?: typeof fetch;
}

/**
 * Free models available through OpenCode / OpenRouter compatible inference gateways
 */
export const OPENCODE_FREE_MODELS: Array<{ id: string; name: string; context: number; reasoning: boolean }> = [
  { id: 'google/gemini-2.0-flash-exp:free', name: 'Google Gemini 2.0 Flash (Free)', context: 1048576, reasoning: true },
  { id: 'meta-llama/llama-3.3-70b-instruct:free', name: 'Llama 3.3 70B Instruct (Free)', context: 131072, reasoning: false },
  { id: 'deepseek/deepseek-r1:free', name: 'DeepSeek R1 (Free)', context: 16384, reasoning: true },
  { id: 'qwen/qwen-2.5-coder-32b-instruct:free', name: 'Qwen 2.5 Coder 32B (Free)', context: 32768, reasoning: false },
  { id: 'mistralai/mistral-7b-instruct:free', name: 'Mistral 7B Instruct (Free)', context: 32768, reasoning: false },
];

export class OpenCodeAdapter implements ProviderAdapter {
  public id = 'opencode';
  public name = 'OpenCode Model Gateway';

  private baseUrl: string;
  private apiKey?: string;
  private customFetch?: typeof fetch;

  constructor(options?: OpenCodeAdapterOptions) {
    this.baseUrl = options?.baseUrl || 'https://openrouter.ai/api/v1';
    this.apiKey = options?.apiKey;
    this.customFetch = options?.fetchFn;
  }

  private get fetchImpl(): typeof fetch {
    return this.customFetch || globalThis.fetch;
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      const res = await this.fetchImpl(`${this.baseUrl}/models`, {
        method: 'GET',
        headers: this.apiKey ? { Authorization: `Bearer ${this.apiKey}` } : {},
        signal: controller.signal,
      }).finally(() => clearTimeout(timeoutId));

      if (res.ok) return 'HEALTHY';
      if (res.status === 401 || res.status === 403) return 'AUTH_ERROR';
      if (res.status === 429) return 'RATE_LIMITED';
      return 'DEGRADED';
    } catch {
      return 'OFFLINE';
    }
  }

  public async listModels(): Promise<ModelDescriptor[]> {
    const health = await this.healthCheck();

    // Map built-in free models
    const freeDescriptors: ModelDescriptor[] = OPENCODE_FREE_MODELS.map((m) => ({
      providerId: this.id,
      modelId: m.id,
      displayName: m.name,
      capabilities: this.getCapabilities(m.id),
      health,
    }));

    try {
      const res = await this.fetchImpl(`${this.baseUrl}/models`);
      if (!res.ok) return freeDescriptors;

      const data = (await res.json()) as { data?: Array<{ id: string; name?: string }> };
      const remoteModels = data.data || [];

      const fetchedDescriptors: ModelDescriptor[] = remoteModels
        .filter((m) => m.id.endsWith(':free') || m.id.includes('opencode'))
        .map((m) => ({
          providerId: this.id,
          modelId: m.id,
          displayName: m.name || m.id,
          capabilities: this.getCapabilities(m.id),
          health,
        }));

      // Combine free descriptors without duplicate IDs
      const combined = [...freeDescriptors];
      for (const fd of fetchedDescriptors) {
        if (!combined.some((c) => c.modelId === fd.modelId)) {
          combined.push(fd);
        }
      }

      return combined;
    } catch {
      return freeDescriptors;
    }
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    const match = OPENCODE_FREE_MODELS.find((m) => m.id === modelId);
    return {
      supportsTools: true,
      supportsVision: modelId.includes('vision') || modelId.includes('gemini'),
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: match ? match.reasoning : modelId.includes('r1') || modelId.includes('reasoning'),
      contextWindow: match ? match.context : 65536,
      maxOutputTokens: 8192,
      isLocal: false,
      isFree: modelId.endsWith(':free') || modelId.includes('free'),
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    const key = credentials.apiKey || this.apiKey;
    if (!key) return true; // Free public endpoints can run without API keys

    try {
      const res = await this.fetchImpl(`${this.baseUrl}/models`, {
        headers: { Authorization: `Bearer ${key}` },
      });
      return res.ok;
    } catch {
      return false;
    }
  }

  public normalizeError(error: any): NormalizedError {
    return ErrorNormalizer.normalize(error, this.id);
  }

  public async chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse> {
    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
        'X-Title': 'OCTREX CODE V4',
      };
      if (this.apiKey) {
        headers['Authorization'] = `Bearer ${this.apiKey}`;
      }

      const body = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: false,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers,
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        const errJson = await res.json().catch(() => ({}));
        throw { status: res.status, message: (errJson as any).error?.message || `OpenCode Gateway HTTP ${res.status}` };
      }

      const json = (await res.json()) as any;
      const choice = json.choices?.[0];

      return {
        id: json.id || `opencode-${Date.now()}`,
        providerId: this.id,
        modelId: request.modelId,
        message: {
          role: 'assistant',
          content: choice?.message?.content || '',
        },
        finishReason: choice?.finish_reason || 'stop',
        usage: json.usage
          ? {
              promptTokens: json.usage.prompt_tokens || 0,
              completionTokens: json.usage.completion_tokens || 0,
              totalTokens: json.usage.total_tokens || 0,
            }
          : undefined,
      };
    } catch (err) {
      throw this.normalizeError(err);
    }
  }

  public async *stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent> {
    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
        'X-Title': 'OCTREX CODE V4',
      };
      if (this.apiKey) {
        headers['Authorization'] = `Bearer ${this.apiKey}`;
      }

      const body = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: true,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers,
        body: JSON.stringify(body),
      });

      if (!res.ok || !res.body) {
        yield {
          type: 'error',
          error: this.normalizeError({ status: res.status, message: 'OpenCode streaming request failed' }),
        };
        return;
      }

      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          const trimmed = line.trim();
          if (!trimmed || trimmed.startsWith(':')) continue;
          if (trimmed === 'data: [DONE]') break;

          if (trimmed.startsWith('data: ')) {
            try {
              const parsed = JSON.parse(trimmed.slice(6));
              const delta = parsed.choices?.[0]?.delta?.content;
              if (delta) {
                yield {
                  type: 'text_delta',
                  delta,
                };
              }
            } catch {
              // Ignore partial JSON
            }
          }
        }
      }

      yield { type: 'finish', finishReason: 'stop' };
    } catch (err) {
      yield {
        type: 'error',
        error: this.normalizeError(err),
      };
    }
  }
}
