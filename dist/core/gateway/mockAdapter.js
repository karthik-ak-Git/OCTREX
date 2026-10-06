import { ErrorNormalizer } from './errorNormalizer.js';
export class MockProviderAdapter {
    id = 'mock-provider';
    name = 'Mock Provider Adapter';
    health = 'HEALTHY';
    shouldFailWithCode;
    mockDelayMs = 0;
    constructor(options) {
        if (options?.initialHealth)
            this.health = options.initialHealth;
        if (options?.shouldFailWithCode)
            this.shouldFailWithCode = options.shouldFailWithCode;
        if (options?.delayMs)
            this.mockDelayMs = options.delayMs;
    }
    setHealth(health) {
        this.health = health;
    }
    setFailureCode(code) {
        this.shouldFailWithCode = code;
    }
    async listModels() {
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
    async healthCheck() {
        return this.health;
    }
    getCapabilities(modelId) {
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
    async validateCredentials(credentials) {
        return Boolean(credentials.apiKey || credentials.endpointUrl);
    }
    normalizeError(error) {
        return ErrorNormalizer.normalize(error, this.id);
    }
    async chat(request) {
        if (this.mockDelayMs > 0) {
            await new Promise((r) => setTimeout(r, this.mockDelayMs));
        }
        if (this.shouldFailWithCode) {
            const err = new Error(`Mock provider HTTP error ${this.shouldFailWithCode}`);
            err.status = this.shouldFailWithCode;
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
    async *stream(request) {
        if (this.mockDelayMs > 0) {
            await new Promise((r) => setTimeout(r, this.mockDelayMs));
        }
        if (this.shouldFailWithCode) {
            const err = new Error(`Mock provider stream HTTP error ${this.shouldFailWithCode}`);
            err.status = this.shouldFailWithCode;
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
//# sourceMappingURL=mockAdapter.js.map