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

export class StreamingToolMockAdapter implements ProviderAdapter {
  public id = 'streaming-tool-mock';
  public name = 'Streaming Tool Mock Adapter';

  private toolChunks: StreamEvent[] = [];

  constructor(customToolChunks?: StreamEvent[]) {
    if (customToolChunks) {
      this.toolChunks = customToolChunks;
    } else {
      // Default: Fragmented tool call simulation for file_patch
      this.toolChunks = [
        {
          type: 'tool_call_delta',
          toolCallDelta: { index: 0, id: 'call_1', name: 'file_patch', argumentsDelta: '{"filePath": "' },
        },
        {
          type: 'tool_call_delta',
          toolCallDelta: { index: 0, argumentsDelta: 'src/index.ts", "startLine": 1, ' },
        },
        {
          type: 'tool_call_delta',
          toolCallDelta: { index: 0, argumentsDelta: '"endLine": 5, "replacementContent": "export * from \\"./app\\";"}' },
        },
        {
          type: 'finish',
          finishReason: 'tool_calls',
        },
      ];
    }
  }

  public setChunks(chunks: StreamEvent[]): void {
    this.toolChunks = chunks;
  }

  public async listModels(): Promise<ModelDescriptor[]> {
    return [
      {
        providerId: this.id,
        modelId: 'tool-mock-model',
        displayName: 'Tool Mock Model',
        capabilities: this.getCapabilities('tool-mock-model'),
        health: 'HEALTHY',
      },
    ];
  }

  public async healthCheck(): Promise<ProviderHealthStatus> {
    return 'HEALTHY';
  }

  public getCapabilities(modelId: string): ModelCapabilities {
    return {
      supportsTools: true,
      supportsVision: false,
      supportsStreaming: true,
      supportsStructuredOutput: true,
      supportsReasoning: false,
      contextWindow: 65536,
      maxOutputTokens: 4096,
      isLocal: false,
      isFree: true,
    };
  }

  public async validateCredentials(credentials: ProviderCredentials): Promise<boolean> {
    return true;
  }

  public normalizeError(error: any): NormalizedError {
    return ErrorNormalizer.normalize(error, this.id);
  }

  public async chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse> {
    return {
      id: 'resp-1',
      providerId: this.id,
      modelId: request.modelId,
      message: {
        role: 'assistant',
        content: '',
        toolCalls: [
          {
            id: 'call_1',
            name: 'file_patch',
            arguments: { filePath: 'src/index.ts', startLine: 1, endLine: 5, replacementContent: 'export * from "./app";' },
          },
        ],
      },
      finishReason: 'tool_calls',
    };
  }

  public async *stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent> {
    for (const chunk of this.toolChunks) {
      yield chunk;
    }
  }
}
