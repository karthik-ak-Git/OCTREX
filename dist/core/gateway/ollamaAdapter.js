import { ErrorNormalizer } from './errorNormalizer.js';
export class OllamaAdapter {
    id = 'ollama';
    name = 'Ollama Local Adapter';
    endpointUrl;
    probeTimeoutMs;
    customFetch;
    isBootstrapped = false;
    cachedHealth = 'UNKNOWN';
    constructor(options) {
        this.endpointUrl = options?.endpointUrl || 'http://127.0.0.1:11434';
        this.probeTimeoutMs = options?.probeTimeoutMs || 2000;
        this.customFetch = options?.fetchFn;
    }
    get fetchImpl() {
        return this.customFetch || globalThis.fetch;
    }
    /**
     * Lazily checks Ollama health without blocking constructor or initial gateway load.
     */
    async healthCheck() {
        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), this.probeTimeoutMs);
            const res = await this.fetchImpl(`${this.endpointUrl}/api/tags`, {
                method: 'GET',
                signal: controller.signal,
            }).finally(() => clearTimeout(timeoutId));
            if (res.ok) {
                this.cachedHealth = 'HEALTHY';
                this.isBootstrapped = true;
                return 'HEALTHY';
            }
            this.cachedHealth = 'DEGRADED';
            return 'DEGRADED';
        }
        catch (err) {
            // Lazy startup handling: Server offline or booting up
            this.cachedHealth = 'OFFLINE';
            return 'OFFLINE';
        }
    }
    async listModels() {
        const health = await this.healthCheck();
        if (health === 'OFFLINE') {
            return [];
        }
        try {
            const res = await this.fetchImpl(`${this.endpointUrl}/api/tags`);
            if (!res.ok)
                return [];
            const data = (await res.json());
            const models = data.models || [];
            return models.map((m) => ({
                providerId: this.id,
                modelId: m.name,
                displayName: `Ollama (${m.name})`,
                capabilities: this.getCapabilities(m.name),
                health,
            }));
        }
        catch (err) {
            return [];
        }
    }
    getCapabilities(modelId) {
        return {
            supportsTools: modelId.includes('llama3') || modelId.includes('qwen') || modelId.includes('mistral'),
            supportsVision: modelId.includes('llava') || modelId.includes('vision'),
            supportsStreaming: true,
            supportsStructuredOutput: true,
            supportsReasoning: modelId.includes('deepseek'),
            contextWindow: 32768,
            maxOutputTokens: 4096,
            isLocal: true,
            isFree: true,
        };
    }
    async validateCredentials(credentials) {
        const url = credentials.endpointUrl || this.endpointUrl;
        try {
            const res = await this.fetchImpl(`${url}/api/tags`);
            return res.ok;
        }
        catch {
            return false;
        }
    }
    normalizeError(error) {
        return ErrorNormalizer.normalize(error, this.id);
    }
    async chat(request) {
        const health = await this.healthCheck();
        if (health === 'OFFLINE') {
            throw this.normalizeError({
                status: 503,
                message: `Ollama service at ${this.endpointUrl} is offline or starting up.`,
            });
        }
        try {
            const payload = {
                model: request.modelId,
                messages: request.messages.map((m) => ({
                    role: m.role,
                    content: m.content,
                })),
                stream: false,
            };
            const res = await this.fetchImpl(`${this.endpointUrl}/api/chat`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });
            if (!res.ok) {
                throw new Error(`Ollama HTTP error ${res.status}`);
            }
            const json = (await res.json());
            return {
                id: `ollama-${Date.now()}`,
                providerId: this.id,
                modelId: request.modelId,
                message: {
                    role: 'assistant',
                    content: json.message?.content || '',
                },
                finishReason: 'stop',
            };
        }
        catch (err) {
            throw this.normalizeError(err);
        }
    }
    async *stream(request) {
        const health = await this.healthCheck();
        if (health === 'OFFLINE') {
            yield {
                type: 'error',
                error: this.normalizeError({
                    status: 503,
                    message: `Ollama service at ${this.endpointUrl} is offline or starting up.`,
                }),
            };
            return;
        }
        try {
            const payload = {
                model: request.modelId,
                messages: request.messages.map((m) => ({ role: m.role, content: m.content })),
                stream: true,
            };
            const res = await this.fetchImpl(`${this.endpointUrl}/api/chat`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });
            if (!res.ok || !res.body) {
                yield {
                    type: 'error',
                    error: this.normalizeError({ status: res.status, message: 'Failed to start Ollama stream' }),
                };
                return;
            }
            const reader = res.body.getReader();
            const decoder = new TextDecoder();
            let buffer = '';
            while (true) {
                const { done, value } = await reader.read();
                if (done)
                    break;
                buffer += decoder.decode(value, { stream: true });
                const lines = buffer.split('\n');
                buffer = lines.pop() || '';
                for (const line of lines) {
                    if (!line.trim())
                        continue;
                    try {
                        const parsed = JSON.parse(line);
                        if (parsed.message?.content) {
                            yield {
                                type: 'text_delta',
                                delta: parsed.message.content,
                            };
                        }
                    }
                    catch {
                        // Ignore partial lines
                    }
                }
            }
            yield { type: 'finish', finishReason: 'stop' };
        }
        catch (err) {
            yield {
                type: 'error',
                error: this.normalizeError(err),
            };
        }
    }
}
//# sourceMappingURL=ollamaAdapter.js.map