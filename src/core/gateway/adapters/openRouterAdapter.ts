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

export interface OpenRouterAdapterOptions {
  apiKey?: string;
  baseUrl?: string;
  fetchFn?: typeof fetch;
}

export class OpenRouterAdapter implements ProviderAdapter {
  public id = 'openrouter';
  public name = 'OpenRouter Gateway';

  private apiKey?: string;
  private baseUrl: string;
  private customFetch?: typeof fetch;

  constructor(options?: OpenRouterAdapterOptions) {
    this.apiKey = options?.apiKey;
    this.baseUrl = options?.baseUrl || 'https://openrouter.ai/api/v1';
    this.customFetch = options?.fetchFn;
  }

  private get fetchImpl(): typeof fetch {
    return this.customFetch || globalThis.fetch;
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    if (!this.apiKey) {
      return 'AUTH_ERROR';
    }

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      const res = await this.fetchImpl(`${this.baseUrl}/auth/key`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${this.apiKey}` },
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

  public async listModels(): Promise<ModelDescriptor[]> {
    const health = await this.healthCheck();

    try {
      const res = await this.fetchImpl(`${this.baseUrl}/models`);
      if (!res.ok) throw new Error(`OpenRouter listModels HTTP ${res.status}`);

      const data = (await res.json()) as { data?: Array<{ id: string; name?: string; context_length?: number }> };
      const models = data.data || [];

      return models.map((m) => ({
        providerId: this.id,
        modelId: m.id,
        displayName: m.name || m.id,
        capabilities: {
          supportsTools: true,
          supportsVision: m.id.includes('vision') || m.id.includes('4o') || m.id.includes('gemini'),
          supportsStreaming: true,
          supportsStructuredOutput: true,
          supportsReasoning: m.id.includes('r1') || m.id.includes('o1') || m.id.includes('o3') || m.id.includes('reasoning'),
          contextWindow: m.context_length || 128000,
          maxOutputTokens: 8192,
          isLocal: false,
          isFree: m.id.endsWith(':free'),
        },
        health,
      }));
    } catch {
      return [
        {
          providerId: this.id,
          modelId: 'anthropic/claude-3.5-sonnet',
          displayName: 'Claude 3.5 Sonnet',
          capabilities: this.getCapabilities('anthropic/claude-3.5-sonnet'),
          health,
        },
      ];
    }
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    return {
      supportsTools: true,
      supportsVision: modelId.includes('vision') || modelId.includes('4o') || modelId.includes('claude') || modelId.includes('gemini'),
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: modelId.includes('r1') || modelId.includes('o1') || modelId.includes('o3') || modelId.includes('reasoning'),
      contextWindow: 128000,
      maxOutputTokens: 8192,
      isLocal: false,
      isFree: modelId.endsWith(':free'),
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    const key = credentials.apiKey || this.apiKey;
    if (!key) return false;

    try {
      const res = await this.fetchImpl(`${this.baseUrl}/auth/key`, {
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
    if (!this.apiKey) {
      throw this.normalizeError({ status: 401, message: 'OpenRouter API key is missing' });
    }

    try {
      const payload = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        tools: request.tools?.map((t) => ({
          type: 'function',
          function: {
            name: t.name,
            description: t.description,
            parameters: t.parameters,
          },
        })),
        stream: false,
      };

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
        const errJson = await res.json().catch(() => ({}));
        throw { status: res.status, message: (errJson as any).error?.message || `OpenRouter API HTTP ${res.status}` };
      }

      const json = (await res.json()) as any;
      const choice = json.choices?.[0];

      return {
        id: json.id || `or-${Date.now()}`,
        providerId: this.id,
        modelId: request.modelId,
        message: {
          role: 'assistant',
          content: choice?.message?.content || '',
          toolCalls: choice?.message?.tool_calls?.map((tc: any) => ({
            id: tc.id,
            name: tc.function.name,
            arguments: JSON.parse(tc.function.arguments || '{}'),
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
        error: this.normalizeError({ status: 401, message: 'OpenRouter API key is missing' }),
      };
      return;
    }

    try {
      const payload = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        tools: request.tools?.map((t) => ({
          type: 'function',
          function: { name: t.name, description: t.description, parameters: t.parameters },
        })),
        stream: true,
      };

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
        yield {
          type: 'error',
          error: this.normalizeError({ status: res.status, message: 'OpenRouter streaming request failed' }),
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
              // Partial JSON
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
