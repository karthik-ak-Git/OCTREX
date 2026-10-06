import { ProviderAdapter, ModelDescriptor, ProviderHealthStatus, NormalizedChatRequest, NormalizedChatResponse, StreamEvent, ModelCapabilities, ProviderCredentials, NormalizedError } from '../types/gateway.js';
export declare class MockProviderAdapter implements ProviderAdapter {
    id: string;
    name: string;
    private health;
    private shouldFailWithCode?;
    private mockDelayMs;
    constructor(options?: {
        initialHealth?: ProviderHealthStatus;
        shouldFailWithCode?: number;
        delayMs?: number;
    });
    setHealth(health: ProviderHealthStatus): void;
    setFailureCode(code?: number): void;
    listModels(): Promise<ModelDescriptor[]>;
    healthCheck(): Promise<ProviderHealthStatus>;
    getCapabilities(modelId: string): ModelCapabilities;
    validateCredentials(credentials: ProviderCredentials): Promise<boolean>;
    normalizeError(error: any): NormalizedError;
    chat(request: NormalizedChatRequest): Promise<NormalizedChatResponse>;
    stream(request: NormalizedChatRequest): AsyncIterable<StreamEvent>;
}
//# sourceMappingURL=mockAdapter.d.ts.map