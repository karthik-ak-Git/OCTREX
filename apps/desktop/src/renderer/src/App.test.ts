import { describe, it, expect } from 'vitest';
import { providerDefinitions, PROVIDER_REGISTRY } from '../../shared/provider-registry';

describe('OCTREX CODE V4 Desktop App', () => {
  it('loads provider registry with all required AI providers', () => {
    expect(providerDefinitions).toBeDefined();
    expect(providerDefinitions.length).toBeGreaterThanOrEqual(8);

    const ids = providerDefinitions.map((p) => p.id);
    expect(ids).toContain('google');
    expect(ids).toContain('nvidia');
    expect(ids).toContain('openai');
    expect(ids).toContain('openrouter');
    expect(ids).toContain('groq');
    expect(ids).toContain('ollama');
    expect(ids).toContain('crax-gpt');
    expect(ids).toContain('cerebras');
    expect(ids).toContain('sambanova');
    expect(ids).toContain('cloudflare');
  });

  it('contains valid provider configurations with name, baseUrl, and testStrategy', () => {
    for (const provider of providerDefinitions) {
      expect(provider.name).toBeTruthy();
      expect(provider.baseUrl).toBeTruthy();
      expect(['openai-compatible', 'cloudflare-workers-ai', 'ollama-local']).toContain(
        provider.testStrategy
      );
    }
  });

  it('provides default models for cloud and local inference', () => {
    expect(PROVIDER_REGISTRY.google.defaultModel).toBe('gemini-2.5-flash');
    expect(PROVIDER_REGISTRY.nvidia.defaultModel).toBe('meta/llama-3.3-70b-instruct');
    expect(PROVIDER_REGISTRY.ollama.defaultModel).toBe('qwen2.5-coder:7b');
  });
});
