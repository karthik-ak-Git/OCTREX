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
} from '../../types/gateway.js';
import { ErrorNormalizer } from '../errorNormalizer.js';

export interface CraxGptAdapterOptions {
  apiKey?: string;
  baseUrl?: string;
  fetchFn?: typeof fetch;
  setupUrl?: string;
}

/**
 * Built-in baseline model catalog for crax-gpt fallback before first dynamic discovery.
 */
export const CRAX_GPT_DEFAULT_MODELS: Array<{ id: string; name: string; context: number; reasoning: boolean; vision: boolean }> = [
  { id: 'glm-5.3', name: 'GLM-5.3 (Reasoning & Code)', context: 131072, reasoning: true, vision: true },
  { id: 'gpt-4o', name: 'GPT-4o (Omni)', context: 128000, reasoning: false, vision: true },
  { id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet', context: 200000, reasoning: true, vision: true },
  { id: 'deepseek-r1', name: 'DeepSeek R1', context: 65536, reasoning: true, vision: false },
  { id: 'qwen-2.5-coder-32b', name: 'Qwen 2.5 Coder 32B', context: 32768, reasoning: false, vision: false },
];

export class CraxGptAdapter implements ProviderAdapter {
  public id = 'crax-gpt';
  public name = 'crax-gpt';
  public setupUrl: string;

  private baseUrl: string;
  private apiKey?: string;
  private customFetch?: typeof fetch;
  private cachedModels: ModelDescriptor[] = [];
  private lastDiscoveryTime: number = 0;

  constructor(options?: CraxGptAdapterOptions) {
    this.apiKey = options?.apiKey;
    this.baseUrl = (options?.baseUrl || 'https://gpt.crax.lol/v1').replace(/\/+$/, '');
    this.setupUrl = options?.setupUrl || 'https://gpt.crax.lol';
    this.customFetch = options?.fetchFn;
  }

  private get fetchImpl(): typeof fetch {
    return this.customFetch || globalThis.fetch;
  }

  public getBaseUrl(): string {
    return this.baseUrl;
  }

  public getSetupUrl(): string {
    return this.setupUrl;
  }

  public setApiKey(key: string): void {
    this.apiKey = key;
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    if (!this.apiKey) {
      return 'AUTH_ERROR';
    }

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      const res = await this.fetchImpl(`${this.baseUrl}/models`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${this.apiKey}`,
          'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
          'X-Title': 'OCTREX CODE V4',
        },
        signal: controller.signal,
      }).finally(() => clearTimeout(timeoutId));

      if (res.ok) return 'HEALTHY';
      if (res.status === 401 || res.status === 403) return 'AUTH_ERROR';
      if (res.status === 429) return 'RATE_LIMITED';
      return 'DEGRADED';
    } catch (err: any) {
      if (err.name === 'AbortError' || String(err).includes('fetch failed')) {
        return 'OFFLINE';
      }
      return 'DEGRADED';
    }
  }

  /**
   * Discovers and returns all models dynamically available on crax-gpt /v1/models endpoint.
   * Updates internal memory cache.
   */
  public async refreshModels(): Promise<ModelDescriptor[]> {
    const health = await this.healthCheck();
    if (health === 'OFFLINE' || health === 'AUTH_ERROR') {
      if (this.cachedModels.length > 0) {
        return this.cachedModels.map((m) => ({ ...m, health }));
      }
      return CRAX_GPT_DEFAULT_MODELS.map((m) => ({
        providerId: this.id,
        modelId: m.id,
        displayName: m.name,
        capabilities: this.getCapabilities(m.id),
        health,
      }));
    }

    try {
      const headers: Record<string, string> = {
        'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
        'X-Title': 'OCTREX CODE V4',
      };
      if (this.apiKey) {
        headers['Authorization'] = `Bearer ${this.apiKey}`;
      }

      const res = await this.fetchImpl(`${this.baseUrl}/models`, {
        method: 'GET',
        headers,
      });

      if (!res.ok) {
        throw new Error(`crax-gpt dynamic discovery HTTP ${res.status}`);
      }

      const data = (await res.json()) as { data?: Array<{ id: string; name?: string; context_length?: number }> };
      const rawModels = data.data || [];

      if (rawModels.length === 0) {
        // Fallback to default catalog if empty data array
        this.cachedModels = CRAX_GPT_DEFAULT_MODELS.map((m) => ({
          providerId: this.id,
          modelId: m.id,
          displayName: m.name,
          capabilities: this.getCapabilities(m.id),
          health,
        }));
      } else {
        this.cachedModels = rawModels.map((m) => ({
          providerId: this.id,
          modelId: m.id,
          displayName: m.name || m.id,
          capabilities: this.getCapabilities(m.id, m.context_length),
          health,
        }));
      }

      this.lastDiscoveryTime = Date.now();
      return this.cachedModels;
    } catch {
      if (this.cachedModels.length > 0) {
        return this.cachedModels.map((m) => ({ ...m, health }));
      }
      return CRAX_GPT_DEFAULT_MODELS.map((m) => ({
        providerId: this.id,
        modelId: m.id,
        displayName: m.name,
        capabilities: this.getCapabilities(m.id),
        health,
      }));
    }
  }

  public async listModels(): Promise<ModelDescriptor[]> {
    if (this.cachedModels.length > 0 && Date.now() - this.lastDiscoveryTime < 60000) {
      return this.cachedModels;
    }
    return this.refreshModels();
  }

  public getCapabilities(modelId: string, explicitContext?: number): ModelCapabilities {
    const idLower = modelId.toLowerCase();
    const defaultMeta = CRAX_GPT_DEFAULT_MODELS.find((m) => m.id.toLowerCase() === idLower);

    const isVision = defaultMeta?.vision ?? (
      idLower.includes('vision') ||
      idLower.includes('vl') ||
      idLower.includes('4o') ||
      idLower.includes('gemini') ||
      idLower.includes('glm') ||
      idLower.includes('claude')
    );

    const isReasoning = defaultMeta?.reasoning ?? (
      idLower.includes('r1') ||
      idLower.includes('reasoning') ||
      idLower.includes('thinking') ||
      idLower.includes('glm') ||
      idLower.includes('o1') ||
      idLower.includes('o3')
    );

    const context = explicitContext || defaultMeta?.context || (
      idLower.includes('claude') ? 200000 :
      idLower.includes('glm') ? 131072 :
      idLower.includes('4o') ? 128000 :
      idLower.includes('deepseek') ? 65536 : 65536
    );

    return {
      supportsTools: true,
      supportsVision: isVision,
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: isReasoning,
      contextWindow: context,
      maxOutputTokens: 8192,
      isLocal: false,
      isFree: false,
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    const key = credentials.apiKey || this.apiKey;
    if (!key) return false;

    const targetUrl = (credentials.endpointUrl || this.baseUrl).replace(/\/+$/, '');
    try {
      const res = await this.fetchImpl(`${targetUrl}/models`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${key}`,
          'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
          'X-Title': 'OCTREX CODE V4',
        },
      });
      return res.ok;
    } catch {
      return false;
    }
  }

  public normalizeError(error: any): NormalizedError {
    const normalized = ErrorNormalizer.normalize(error, this.id);
    
    // Check for Retry-After header metadata if present
    if (error?.retryAfter !== undefined) {
      normalized.message = `${normalized.message} (Retry after ${error.retryAfter}s)`;
    }
    
    return normalized;
  }

  public async chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse> {
    if (!this.apiKey) {
      throw this.normalizeError({ status: 401, message: 'crax-gpt API key is required. Connect your key in Providers Settings.' });
    }

    try {
      const payload: Record<string, any> = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: false,
      };

      if (request.tools && request.tools.length > 0) {
        payload.tools = request.tools.map((t) => ({
          type: 'function',
          function: {
            name: t.name,
            description: t.description,
            parameters: t.parameters,
          },
        }));
      }

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${this.apiKey}`,
          'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
          'X-Title': 'OCTREX CODE V4',
        },
        body: JSON.stringify(payload),
      });

      if (!res.ok) {
        const retryAfterHeader = res.headers?.get?.('retry-after');
        const errJson = await res.json().catch(() => ({}));
        const rawErrMsg = (errJson as any)?.error?.message || `crax-gpt HTTP ${res.status}`;
        
        throw {
          status: res.status,
          message: rawErrMsg,
          retryAfter: retryAfterHeader ? parseInt(retryAfterHeader, 10) || retryAfterHeader : undefined,
        };
      }

      const json = (await res.json()) as any;
      const choice = json.choices?.[0];

      return {
        id: json.id || `crax-${Date.now()}`,
        providerId: this.id,
        modelId: request.modelId,
        message: {
          role: 'assistant',
          content: choice?.message?.content || '',
          toolCalls: choice?.message?.tool_calls?.map((tc: any) => ({
            id: tc.id,
            name: tc.function?.name,
            arguments: typeof tc.function?.arguments === 'string'
              ? JSON.parse(tc.function.arguments || '{}')
              : tc.function?.arguments || {},
          })),
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
    if (!this.apiKey) {
      yield {
        type: 'error',
        error: this.normalizeError({ status: 401, message: 'crax-gpt API key is required. Connect your key in Providers Settings.' }),
      };
      return;
    }

    try {
      const payload: Record<string, any> = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: true,
      };

      if (request.tools && request.tools.length > 0) {
        payload.tools = request.tools.map((t) => ({
          type: 'function',
          function: {
            name: t.name,
            description: t.description,
            parameters: t.parameters,
          },
        }));
      }

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${this.apiKey}`,
          'HTTP-Referer': 'https://github.com/karthik-ak-Git/OCTREX',
          'X-Title': 'OCTREX CODE V4',
        },
        body: JSON.stringify(payload),
      });

      if (!res.ok || !res.body) {
        const retryAfterHeader = res.headers?.get?.('retry-after');
        const errJson = await res.json().catch(() => ({}));
        const rawErrMsg = (errJson as any)?.error?.message || `crax-gpt streaming failed with HTTP ${res.status}`;

        yield {
          type: 'error',
          error: this.normalizeError({
            status: res.status,
            message: rawErrMsg,
            retryAfter: retryAfterHeader ? parseInt(retryAfterHeader, 10) || retryAfterHeader : undefined,
          }),
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
              const delta = parsed.choices?.[0]?.delta;
              if (delta?.content) {
                yield { type: 'text_delta', delta: delta.content };
              }
              if (delta?.tool_calls) {
                for (const tc of delta.tool_calls) {
                  yield {
                    type: 'tool_call_delta',
                    toolCallDelta: {
                      index: tc.index ?? 0,
                      id: tc.id,
                      name: tc.function?.name,
                      argumentsDelta: tc.function?.arguments,
                    },
                  };
                }
              }
            } catch {
              // Partial JSON in stream chunk
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
