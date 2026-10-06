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

export interface OpenAICompatibleAdapterOptions {
  id?: string;
  name?: string;
  baseUrl: string;
  apiKey?: string;
  fetchFn?: typeof fetch;
}

export class OpenAICompatibleAdapter implements ProviderAdapter {
  public id: string;
  public name: string;

  private baseUrl: string;
  private apiKey?: string;
  private customFetch?: typeof fetch;

  constructor(options: OpenAICompatibleAdapterOptions) {
    this.id = options.id || 'openai-compatible';
    this.name = options.name || 'Custom OpenAI Endpoint';
    this.baseUrl = options.baseUrl.replace(/\/+$/, '');
    this.apiKey = options.apiKey;
    this.customFetch = options.fetchFn;
  }

  private get fetchImpl(): typeof fetch {
    return this.customFetch || globalThis.fetch;
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      const headers: Record<string, string> = {};
      if (this.apiKey) headers['Authorization'] = `Bearer ${this.apiKey}`;

      const res = await this.fetchImpl(`${this.baseUrl}/models`, {
        method: 'GET',
        headers,
        signal: controller.signal,
      }).finally(() => clearTimeout(timeoutId));

      if (res.ok) return 'HEALTHY';
      if (res.status === 401 || res.status === 403) return 'AUTH_ERROR';
      if (res.status === 429) return 'RATE_LIMITED';
      return 'DEGRADED';
    } catch {
      // Bad base URL or unreachable endpoint
      return 'OFFLINE';
    }
  }

  public async listModels(): Promise<ModelDescriptor[]> {
    const health = await this.healthCheck();
    if (health === 'OFFLINE') return [];

    try {
      const headers: Record<string, string> = {};
      if (this.apiKey) headers['Authorization'] = `Bearer ${this.apiKey}`;

      const res = await this.fetchImpl(`${this.baseUrl}/models`, { headers });
      if (!res.ok) return [];

      const data = (await res.json()) as { data?: Array<{ id: string }> };
      const models = data.data || [];

      return models.map((m) => ({
        providerId: this.id,
        modelId: m.id,
        displayName: m.id,
        capabilities: this.getCapabilities(m.id),
        health,
      }));
    } catch {
      return [];
    }
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    return {
      supportsTools: true,
      supportsVision: modelId.includes('vision'),
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: modelId.includes('r1') || modelId.includes('reasoning'),
      contextWindow: 65536,
      maxOutputTokens: 4096,
      isLocal: this.baseUrl.includes('localhost') || this.baseUrl.includes('127.0.0.1'),
      isFree: false,
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    const key = credentials.apiKey || this.apiKey;
    const url = credentials.endpointUrl || this.baseUrl;

    try {
      const res = await this.fetchImpl(`${url}/models`, {
        headers: key ? { Authorization: `Bearer ${key}` } : {},
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
      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
      if (this.apiKey) headers['Authorization'] = `Bearer ${this.apiKey}`;

      const payload = {
        model: request.modelId,
        messages: request.messages,
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: false,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers,
        body: JSON.stringify(payload),
      });

      if (!res.ok) throw { status: res.status, message: `Custom OpenAI HTTP ${res.status}` };

      const json = (await res.json()) as any;
      return {
        id: json.id || `custom-${Date.now()}`,
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
    try {
      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
      if (this.apiKey) headers['Authorization'] = `Bearer ${this.apiKey}`;

      const payload = {
        model: request.modelId,
        messages: request.messages,
        stream: true,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers,
        body: JSON.stringify(payload),
      });

      if (!res.ok || !res.body) {
        yield { type: 'error', error: this.normalizeError({ status: res.status, message: 'Custom OpenAI stream failed' }) };
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
