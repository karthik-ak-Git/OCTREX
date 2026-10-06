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

export interface GroqAdapterOptions {
  apiKey?: string;
  baseUrl?: string;
  fetchFn?: typeof fetch;
}

export class GroqAdapter implements ProviderAdapter {
  public id = 'groq';
  public name = 'Groq Cloud';

  private apiKey?: string;
  private baseUrl: string;
  private customFetch?: typeof fetch;

  constructor(options?: GroqAdapterOptions) {
    this.apiKey = options?.apiKey;
    this.baseUrl = options?.baseUrl || 'https://api.groq.com/openai/v1';
    this.customFetch = options?.fetchFn;
  }

  private get fetchImpl(): typeof fetch {
    return this.customFetch || globalThis.fetch;
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    if (!this.apiKey) return 'AUTH_ERROR';

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      const res = await this.fetchImpl(`${this.baseUrl}/models`, {
        method: 'GET',
        headers: { Authorization: `Bearer ${this.apiKey}` },
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
    return [
      {
        providerId: this.id,
        modelId: 'llama-3.3-70b-versatile',
        displayName: 'Groq Llama 3.3 70B',
        capabilities: this.getCapabilities('llama-3.3-70b-versatile'),
        health,
      },
      {
        providerId: this.id,
        modelId: 'llama-3.1-8b-instant',
        displayName: 'Groq Llama 3.1 8B Instant',
        capabilities: this.getCapabilities('llama-3.1-8b-instant'),
        health,
      },
    ];
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    return {
      supportsTools: true,
      supportsVision: false,
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: false,
      contextWindow: 131072,
      maxOutputTokens: 8192,
      isLocal: false,
      isFree: false,
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    const key = credentials.apiKey || this.apiKey;
    if (!key) return false;
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
    if (!this.apiKey) {
      throw this.normalizeError({ status: 401, message: 'Groq API key is missing' });
    }

    try {
      const payload = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: false,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${this.apiKey}`,
        },
        body: JSON.stringify(payload),
      });

      if (!res.ok) throw { status: res.status, message: `Groq HTTP ${res.status}` };

      const json = (await res.json()) as any;
      return {
        id: json.id || `groq-${Date.now()}`,
        providerId: this.id,
        modelId: request.modelId,
        message: {
          role: 'assistant',
          content: json.choices?.[0]?.message?.content || '',
        },
        finishReason: 'stop',
      };
    } catch (err) {
      throw this.normalizeError(err);
    }
  }

  public async *stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent> {
    if (!this.apiKey) {
      yield { type: 'error', error: this.normalizeError({ status: 401, message: 'Groq API key is missing' }) };
      return;
    }

    try {
      const payload = {
        model: request.modelId,
        messages: request.messages,
        stream: true,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${this.apiKey}`,
        },
        body: JSON.stringify(payload),
      });

      if (!res.ok || !res.body) {
        yield { type: 'error', error: this.normalizeError({ status: res.status, message: 'Groq stream failed' }) };
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
          if (!trimmed || trimmed === 'data: [DONE]') continue;
          if (trimmed.startsWith('data: ')) {
            try {
              const parsed = JSON.parse(trimmed.slice(6));
              const delta = parsed.choices?.[0]?.delta?.content;
              if (delta) yield { type: 'text_delta', delta };
            } catch {}
          }
        }
      }

      yield { type: 'finish', finishReason: 'stop' };
    } catch (err) {
      yield { type: 'error', error: this.normalizeError(err) };
    }
  }
}
