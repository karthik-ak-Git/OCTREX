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

export interface NvidiaAdapterOptions {
  apiKey?: string;
  baseUrl?: string;
  fetchFn?: typeof fetch;
}

export class NvidiaAdapter implements ProviderAdapter {
  public id = 'nvidia';
  public name = 'NVIDIA NIM';

  private apiKey?: string;
  private baseUrl: string;
  private customFetch?: typeof fetch;

  constructor(options?: NvidiaAdapterOptions) {
    this.apiKey = options?.apiKey;
    this.baseUrl = options?.baseUrl || 'https://integrate.api.nvidia.com/v1';
    this.customFetch = options?.fetchFn;
  }

  private get fetchImpl(): typeof fetch {
    return this.customFetch || globalThis.fetch;
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    if (!this.apiKey && !this.baseUrl.includes('localhost') && !this.baseUrl.includes('127.0.0.1')) {
      return 'AUTH_ERROR';
    }

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
    } catch (err: any) {
      if (err.name === 'AbortError' || String(err).includes('fetch failed')) {
        return 'OFFLINE';
      }
      return 'DEGRADED';
    }
  }

  public async listModels(): Promise<ModelDescriptor[]> {
    const health = await this.healthCheck();
    return [
      {
        providerId: this.id,
        modelId: 'meta/llama-3.1-405b-instruct',
        displayName: 'NVIDIA Llama 3.1 405B',
        capabilities: this.getCapabilities('meta/llama-3.1-405b-instruct'),
        health,
      },
      {
        providerId: this.id,
        modelId: 'meta/llama-3.1-70b-instruct',
        displayName: 'NVIDIA Llama 3.1 70B',
        capabilities: this.getCapabilities('meta/llama-3.1-70b-instruct'),
        health,
      },
      {
        providerId: this.id,
        modelId: 'nvidia/llama-3.1-nemotron-70b-instruct',
        displayName: 'NVIDIA Nemotron 70B',
        capabilities: this.getCapabilities('nvidia/llama-3.1-nemotron-70b-instruct'),
        health,
      },
    ];
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    return {
      supportsTools: true,
      supportsVision: modelId.includes('vision') || modelId.includes('neva'),
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: modelId.includes('nemotron') || modelId.includes('405b'),
      contextWindow: 131072,
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

      if (!res.ok) {
        throw { status: res.status, message: `NVIDIA NIM HTTP ${res.status}` };
      }

      const json = (await res.json()) as any;
      const choice = json.choices?.[0];

      return {
        id: json.id || `nv-${Date.now()}`,
        providerId: this.id,
        modelId: request.modelId,
        message: {
          role: 'assistant',
          content: choice?.message?.content || '',
        },
        finishReason: choice?.finish_reason || 'stop',
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
        temperature: request.temperature ?? 0.2,
        max_tokens: request.maxTokens,
        stream: true,
      };

      const res = await this.fetchImpl(`${this.baseUrl}/chat/completions`, {
        method: 'POST',
        headers,
        body: JSON.stringify(payload),
      });

      if (!res.ok || !res.body) {
        yield { type: 'error', error: this.normalizeError({ status: res.status, message: 'NVIDIA stream failed' }) };
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
