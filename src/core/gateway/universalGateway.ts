import {
  ProviderAdapter,
  ModelDescriptor,
  ProviderHealthStatus,
  NormalizedChatRequest,
  NormalizedChatResponse,
  StreamEvent,
  ModelCapabilities,
  NormalizedError,
} from '../types/gateway.js';
import { StreamingToolParser } from './streamingToolParser.js';
import { ErrorNormalizer } from './errorNormalizer.js';
import { ICancellationToken } from '../types/agent.js';

export class UniversalModelGateway {
  private adapters: Map<string, ProviderAdapter> = new Map();

  public registerAdapter(adapter: ProviderAdapter): void {
    this.adapters.set(adapter.id, adapter);
  }

  public getAdapter(providerId: string): ProviderAdapter | undefined {
    return this.adapters.get(providerId);
  }

  public getAllAdapters(): ProviderAdapter[] {
    return Array.from(this.adapters.values());
  }

  public async checkAllHealth(): Promise<Map<string, ProviderHealthStatus>> {
    const healthMap = new Map<string, ProviderHealthStatus>();
    for (const [id, adapter] of this.adapters.entries()) {
      try {
        const health = await adapter.healthCheck();
        healthMap.set(id, health);
      } catch {
        healthMap.set(id, 'OFFLINE');
      }
    }
    return healthMap;
  }

  public async listAllModels(): Promise<ModelDescriptor[]> {
    const allModels: ModelDescriptor[] = [];
    for (const adapter of this.adapters.values()) {
      try {
        const models = await adapter.listModels();
        allModels.push(...models);
      } catch {
        // Skip offline provider models gracefully
      }
    }
    return allModels;
  }

  public async chat(
    request: NormalizedChatRequest,
    cancellationToken?: ICancellationToken
  ): Promise<NormalizedChatResponse> {
    if (cancellationToken?.isCancelled) {
      throw ErrorNormalizer.normalize(
        new Error(`Chat operation cancelled: ${cancellationToken.cancelReason}`),
        request.providerId,
        request.modelId
      );
    }

    const adapter = this.adapters.get(request.providerId);
    if (!adapter) {
      throw {
        code: 'PROVIDER_UNAVAILABLE',
        message: `Provider '${request.providerId}' is not registered in Universal Gateway`,
        providerId: request.providerId,
        modelId: request.modelId,
        retryable: false,
      } as NormalizedError;
    }

    return adapter.chat(request);
  }

  /**
   * Normalizes streamed tokens and streamed tool call fragments into a clean event stream.
   * Tool calls are only yielded when complete and JSON-valid.
   */
  public async *stream(
    request: NormalizedChatRequest,
    cancellationToken?: ICancellationToken
  ): AsyncIterable<StreamEvent> {
    if (cancellationToken?.isCancelled) {
      yield {
        type: 'error',
        error: ErrorNormalizer.normalize(
          new Error(`Stream operation cancelled: ${cancellationToken.cancelReason}`),
          request.providerId,
          request.modelId
        ),
      };
      return;
    }

    const adapter = this.adapters.get(request.providerId);
    if (!adapter) {
      yield {
        type: 'error',
        error: {
          code: 'PROVIDER_UNAVAILABLE',
          message: `Provider '${request.providerId}' is not registered in Universal Gateway`,
          providerId: request.providerId,
          modelId: request.modelId,
          retryable: false,
        },
      };
      return;
    }

    const parser = new StreamingToolParser(request.providerId, request.modelId);
    let hasToolCallDeltas = false;

    try {
      for await (const chunk of adapter.stream(request)) {
        if (cancellationToken?.isCancelled) {
          yield {
            type: 'error',
            error: ErrorNormalizer.normalize(
              new Error(`Stream cancelled mid-flight: ${cancellationToken.cancelReason}`),
              request.providerId,
              request.modelId
            ),
          };
          return;
        }

        if (chunk.type === 'tool_call_delta' && chunk.toolCallDelta) {
          hasToolCallDeltas = true;
          parser.pushDelta(chunk.toolCallDelta);
          // Do not yield unvalidated fragments directly as tool executions
          continue;
        }

        if (chunk.type === 'finish') {
          if (hasToolCallDeltas) {
            const { toolCalls, error } = parser.finalize();
            if (error) {
              yield { type: 'error', error };
              return;
            }
            if (toolCalls.length > 0) {
              yield {
                type: 'tool_call_complete',
                toolCalls,
                finishReason: 'tool_calls',
              };
            }
          }
          yield chunk;
          return;
        }

        yield chunk;
      }
    } catch (err: any) {
      yield {
        type: 'error',
        error: adapter.normalizeError(err),
      };
    }
  }
}
