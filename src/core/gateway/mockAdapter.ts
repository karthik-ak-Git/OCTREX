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

export class MockProviderAdapter implements ProviderAdapter {
  public id = 'mock-provider';
  public name = 'Mock Provider Adapter';

  private health: ProviderHealthStatus = 'HEALTHY';
  private shouldFailWithCode?: number;
  private mockDelayMs = 0;

  constructor(options?: { initialHealth?: ProviderHealthStatus; shouldFailWithCode?: number; delayMs?: number }) {
    if (options?.initialHealth) this.health = options.initialHealth;
    if (options?.shouldFailWithCode) this.shouldFailWithCode = options.shouldFailWithCode;
    if (options?.delayMs) this.mockDelayMs = options.delayMs;
  }

  public setHealth(health: ProviderHealthStatus) {
    this.health = health;
  }

  public setFailureCode(code?: number) {
    this.shouldFailWithCode = code;
  }

  public async listModels(): Promise<ModelDescriptor[]> {
    return [
      {
        providerId: this.id,
        modelId: 'mock-model-v1',
        displayName: 'Mock General Model',
        capabilities: this.getCapabilities('mock-model-v1'),
        health: this.health,
      },
    ];
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    return this.health;
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    return {
      supportsTools: true,
      supportsVision: true,
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: true,
      contextWindow: 128000,
      maxOutputTokens: 8192,
      isLocal: false,
      isFree: true,
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    return Boolean(credentials.apiKey || credentials.endpointUrl);
  }

  public normalizeError(error: any): NormalizedError {
    return ErrorNormalizer.normalize(error, this.id);
  }

  public async chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse> {
    if (this.mockDelayMs > 0) {
      await new Promise((r) => setTimeout(r, this.mockDelayMs));
    }

    if (this.shouldFailWithCode) {
      const err = new Error(`Mock provider HTTP error ${this.shouldFailWithCode}`);
      (err as any).status = this.shouldFailWithCode;
      throw this.normalizeError(err);
    }

    const userMsg = request.messages[request.messages.length - 1]?.content || '';

    return {
      id: `mock-resp-${Date.now()}`,
      providerId: this.id,
      modelId: request.modelId,
      message: {
        role: 'assistant',
        content: `Mock Response to: ${userMsg}`,
      },
      finishReason: 'stop',
      usage: {
        promptTokens: 10,
        completionTokens: 15,
        totalTokens: 25,
      },
    };
  }

  public async *stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent> {
    if (this.mockDelayMs > 0) {
      await new Promise((r) => setTimeout(r, this.mockDelayMs));
    }

    if (this.shouldFailWithCode) {
      const err = new Error(`Mock provider stream HTTP error ${this.shouldFailWithCode}`);
      (err as any).status = this.shouldFailWithCode;
      yield {
        type: 'error',
        error: this.normalizeError(err),
      };
      return;
    }

    const responseText = `Mock Stream Response to: ${request.messages[request.messages.length - 1]?.content || ''}`;
    const words = responseText.split(' ');

    for (const word of words) {
      yield {
        type: 'text_delta',
        delta: word + ' ',
      };
    }

    yield {
      type: 'finish',
      finishReason: 'stop',
    };
  }
}
