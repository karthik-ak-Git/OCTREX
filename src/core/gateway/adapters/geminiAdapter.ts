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

export interface GeminiAdapterOptions {
  apiKey?: string;
  baseUrl?: string;
  fetchFn?: typeof fetch;
}

export class GeminiAdapter implements ProviderAdapter {
  public id = 'gemini';
  public name = 'Google Gemini API';

  private apiKey?: string;
  private baseUrl: string;
  private customFetch?: typeof fetch;

  constructor(options?: GeminiAdapterOptions) {
    this.apiKey = options?.apiKey;
    this.baseUrl = options?.baseUrl || 'https://generativelanguage.googleapis.com/v1beta';
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

      const res = await this.fetchImpl(`${this.baseUrl}/models?key=${this.apiKey}`, {
        method: 'GET',
        signal: controller.signal,
      }).finally(() => clearTimeout(timeoutId));

      if (res.ok) return 'HEALTHY';
      if (res.status === 400 || res.status === 401 || res.status === 403) return 'AUTH_ERROR';
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
    if (health === 'AUTH_ERROR' || health === 'OFFLINE') {
      return [
        {
          providerId: this.id,
          modelId: 'gemini-2.0-flash',
          displayName: 'Gemini 2.0 Flash',
          capabilities: this.getCapabilities('gemini-2.0-flash'),
          health,
        },
        {
          providerId: this.id,
          modelId: 'gemini-1.5-pro',
          displayName: 'Gemini 1.5 Pro',
          capabilities: this.getCapabilities('gemini-1.5-pro'),
          health,
        },
      ];
    }

    try {
      const res = await this.fetchImpl(`${this.baseUrl}/models?key=${this.apiKey}`);
      if (!res.ok) throw new Error(`Gemini listModels HTTP ${res.status}`);

      const data = (await res.json()) as { models?: Array<{ name: string; displayName?: string }> };
      const models = data.models || [];

      return models
        .filter((m) => m.name.includes('gemini'))
        .map((m) => {
          const rawId = m.name.replace('models/', '');
          return {
            providerId: this.id,
            modelId: rawId,
            displayName: m.displayName || rawId,
            capabilities: this.getCapabilities(rawId),
            health,
          };
        });
    } catch {
      return [
        {
          providerId: this.id,
          modelId: 'gemini-2.0-flash',
          displayName: 'Gemini 2.0 Flash',
          capabilities: this.getCapabilities('gemini-2.0-flash'),
          health,
        },
      ];
    }
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    const isPro = modelId.includes('pro');
    return {
      supportsTools: true,
      supportsVision: true,
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: isPro || modelId.includes('2.0') || modelId.includes('thinking'),
      contextWindow: isPro ? 2097152 : 1048576,
      maxOutputTokens: 8192,
      isLocal: false,
      isFree: false,
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    const key = credentials.apiKey || this.apiKey;
    if (!key) return false;

    try {
      const res = await this.fetchImpl(`${this.baseUrl}/models?key=${key}`);
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
      throw this.normalizeError({ status: 401, message: 'Google Gemini API key is missing' });
    }

    try {
      const targetModel = request.modelId.replace('models/', '');
      const contents = request.messages.map((m) => ({
        role: m.role === 'assistant' ? 'model' : 'user',
        parts: [{ text: m.content }],
      }));

      const payload = { contents };

      const res = await this.fetchImpl(`${this.baseUrl}/models/${targetModel}:generateContent?key=${this.apiKey}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (!res.ok) {
        const errJson = await res.json().catch(() => ({}));
        throw { status: res.status, message: (errJson as any).error?.message || `Gemini API HTTP ${res.status}` };
      }

      const json = (await res.json()) as any;
      const text = json.candidates?.[0]?.content?.parts?.[0]?.text || '';

      return {
        id: `gemini-${Date.now()}`,
        providerId: this.id,
        modelId: request.modelId,
        message: {
          role: 'assistant',
          content: text,
        },
        finishReason: 'stop',
      };
    } catch (err) {
      throw this.normalizeError(err);
    }
  }

  public async *stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent> {
    if (!this.apiKey) {
      yield {
        type: 'error',
        error: this.normalizeError({ status: 401, message: 'Google Gemini API key is missing' }),
      };
      return;
    }

    try {
      const targetModel = request.modelId.replace('models/', '');
      const contents = request.messages.map((m) => ({
        role: m.role === 'assistant' ? 'model' : 'user',
        parts: [{ text: m.content }],
      }));

      const res = await this.fetchImpl(
        `${this.baseUrl}/models/${targetModel}:streamGenerateContent?alt=sse&key=${this.apiKey}`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ contents }),
        }
      );

      if (!res.ok || !res.body) {
        yield {
          type: 'error',
          error: this.normalizeError({ status: res.status, message: 'Gemini stream failed to initialize' }),
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

          if (trimmed.startsWith('data: ')) {
            try {
              const parsed = JSON.parse(trimmed.slice(6));
              const text = parsed.candidates?.[0]?.content?.parts?.[0]?.text;
              if (text) {
                yield {
                  type: 'text_delta',
                  delta: text,
                };
              }
            } catch {
              // Partial SSE chunk
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
