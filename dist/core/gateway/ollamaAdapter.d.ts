import { ProviderAdapter, ModelDescriptor, ProviderHealthStatus, NormalizedChatRequest, NormalizedChatResponse, StreamEvent, ModelCapabilities, ProviderCredentials, NormalizedError } from '../types/gateway.js';
export interface OllamaAdapterOptions {
    endpointUrl?: string;
    probeTimeoutMs?: number;
    fetchFn?: typeof fetch;
}
export declare class OllamaAdapter implements ProviderAdapter {
    id: string;
    name: string;
    private endpointUrl;
    private probeTimeoutMs;
    private customFetch?;
    private isBootstrapped;
    private cachedHealth;
    constructor(options?: OllamaAdapterOptions);
    private get fetchImpl();
    /**
     * Lazily checks Ollama health without blocking constructor or initial gateway load.
     */
    healthCheck(): Promise<ProviderHealthStatus>;
    listModels(): Promise<ModelDescriptor[]>;
    getCapabilities(modelId: string): ModelCapabilities;
    validateCredentials(credentials: ProviderCredentials): Promise<boolean>;
    normalizeError(error: any): NormalizedError;
    chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse>;
    stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent>;
}
//# sourceMappingURL=ollamaAdapter.d.ts.map