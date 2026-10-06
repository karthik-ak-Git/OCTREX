import { ToolCall, StreamToolCallDelta, NormalizedError } from '../types/gateway.js';
import { ErrorNormalizer } from './errorNormalizer.js';

interface InFlightToolCall {
  index: number;
  id: string;
  name: string;
  rawArguments: string;
}

export class StreamingToolParser {
  private inFlightCalls: Map<number, InFlightToolCall> = new Map();
  private providerId: string;
  private modelId?: string;

  constructor(providerId: string, modelId?: string) {
    this.providerId = providerId;
    this.modelId = modelId;
  }

  /**
   * Appends a tool call delta chunk to the in-flight accumulator.
   */
  public pushDelta(delta: StreamToolCallDelta): void {
    const idx = delta.index ?? 0;
    let inFlight = this.inFlightCalls.get(idx);

    if (!inFlight) {
      inFlight = {
        index: idx,
        id: delta.id || `call_${Date.now()}_${idx}`,
        name: delta.name || '',
        rawArguments: '',
      };
      this.inFlightCalls.set(idx, inFlight);
    } else {
      if (delta.id) inFlight.id = delta.id;
      if (delta.name) inFlight.name += delta.name;
    }

    if (delta.argumentsDelta) {
      inFlight.rawArguments += delta.argumentsDelta;
    }
  }

  /**
   * Finalizes all accumulated tool calls and returns fully validated ToolCall objects.
   * Throws or returns NormalizedError if JSON arguments are malformed.
   */
  public finalize(): { toolCalls: ToolCall[]; error?: NormalizedError } {
    const result: ToolCall[] = [];

    for (const [idx, inFlight] of this.inFlightCalls.entries()) {
      let parsedArgs: Record<string, any> = {};
      const raw = inFlight.rawArguments.trim();

      if (raw) {
        try {
          parsedArgs = JSON.parse(raw);
        } catch (jsonErr: any) {
          const normErr: NormalizedError = {
            code: 'MALFORMED_STREAM',
            message: `Malformed JSON in streamed tool call arguments for function '${inFlight.name}': ${jsonErr.message}`,
            providerId: this.providerId,
            modelId: this.modelId,
            rawError: jsonErr,
            retryable: false,
          };
          return { toolCalls: [], error: normErr };
        }
      }

      result.push({
        id: inFlight.id,
        name: inFlight.name || 'unnamed_tool',
        arguments: parsedArgs,
      });
    }

    return { toolCalls: result };
  }

  public reset(): void {
    this.inFlightCalls.clear();
  }
}
