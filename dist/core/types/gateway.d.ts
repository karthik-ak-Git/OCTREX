/**
 * OCTREX CODE V4 - Universal Model Gateway Contracts
 */
export type ProviderHealthStatus = 'HEALTHY' | 'DEGRADED' | 'RATE_LIMITED' | 'AUTH_ERROR' | 'OFFLINE' | 'UNSUPPORTED' | 'UNKNOWN';
export interface ModelCapabilities {
    supportsTools: boolean;
    supportsVision: boolean;
    supportsStreaming: boolean;
    supportsStructuredOutput: boolean;
    supportsReasoning: boolean;
    contextWindow: number;
    maxOutputTokens: number;
    isLocal: boolean;
    isFree: boolean;
}
export interface ModelDescriptor {
    providerId: string;
    modelId: string;
    displayName: string;
    capabilities: ModelCapabilities;
    health: ProviderHealthStatus;
}
export type NormalizedErrorCode = 'INVALID_REQUEST' | 'AUTH_FAILURE' | 'PERMISSION_DENIED' | 'MODEL_NOT_FOUND' | 'TIMEOUT' | 'CONTEXT_EXCEEDED' | 'RATE_LIMITED' | 'PROVIDER_UNAVAILABLE' | 'STREAM_DISCONNECTED' | 'UNKNOWN_ERROR';
export interface NormalizedError {
    code: NormalizedErrorCode;
    message: string;
    httpStatus?: number;
    providerId: string;
    modelId?: string;
    rawError?: any;
    retryable: boolean;
}
export interface ToolDefinition {
    name: string;
    description: string;
    parameters: Record<string, any>;
}
export interface ToolCall {
    id: string;
    name: string;
    arguments: Record<string, any>;
}
export interface ToolResult {
    toolCallId: string;
    name: string;
    output: string;
    error?: string;
    isError: boolean;
}
export interface ChatMessage {
    role: 'system' | 'user' | 'assistant' | 'tool';
    content: string;
    name?: string;
    toolCalls?: ToolCall[];
    toolResults?: ToolResult[];
}
export interface NormalizedChatRequest {
    providerId: string;
    modelId: string;
    messages: ChatMessage[];
    tools?: ToolDefinition[];
    temperature?: number;
    maxTokens?: number;
    topP?: number;
    stopSequences?: string[];
    systemInstruction?: string;
}
export interface NormalizedChatResponse {
    id: string;
    providerId: string;
    modelId: string;
    message: ChatMessage;
    finishReason: 'stop' | 'tool_calls' | 'length' | 'content_filter' | 'error';
    usage?: {
        promptTokens: number;
        completionTokens: number;
        totalTokens: number;
    };
}
export type StreamChunkType = 'text_delta' | 'tool_call_delta' | 'finish' | 'error';
export interface StreamEvent {
    type: StreamChunkType;
    delta?: string;
    toolCall?: Partial<ToolCall>;
    finishReason?: string;
    error?: NormalizedError;
}
export interface ProviderCredentials {
    apiKey?: string;
    endpointUrl?: string;
    organizationId?: string;
}
export interface ProviderAdapter {
    id: string;
    name: string;
    listModels(): Promise<ModelDescriptor[]>;
    healthCheck(): Promise<ProviderHealthStatus>;
    chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse>;
    stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent>;
    getCapabilities(modelId: string): ModelCapabilities;
    validateCredentials(credentials: ProviderCredentials): Promise<boolean>;
    normalizeError(error: any): NormalizedError;
}
//# sourceMappingURL=gateway.d.ts.map