import { test } from 'node:test';
import assert from 'node:assert/strict';
import { OllamaAdapter } from '../core/gateway/ollamaAdapter.js';

test('OllamaAdapter handles lazy startup gracefully when server is offline or booting', async () => {
  // Mock fetch that simulates offline/connection error during startup probe
  const offlineFetch: typeof fetch = async () => {
    throw new Error('fetch failed (ECONNREFUSED)');
  };

  const adapter = new OllamaAdapter({
    endpointUrl: 'http://127.0.0.1:11434',
    probeTimeoutMs: 100,
    fetchFn: offlineFetch,
  });

  // Check health lazily — must return OFFLINE without crashing Gateway
  const health = await adapter.healthCheck();
  assert.equal(health, 'OFFLINE');

  // List models when offline should return empty array safely
  const models = await adapter.listModels();
  assert.equal(models.length, 0);

  // Chat when offline should throw a normalized error rather than raw crash
  await assert.rejects(
    async () => {
      await adapter.chat({
        providerId: 'ollama',
        modelId: 'llama3',
        messages: [{ role: 'user', content: 'test' }],
      });
    },
    (err: any) => {
      return err.code === 'PROVIDER_UNAVAILABLE' && err.message.includes('offline or starting up');
    }
  );
});

test('OllamaAdapter reports HEALTHY when local server responds successfully', async () => {
  const onlineFetch: typeof fetch = async (url) => {
    if (String(url).endsWith('/api/tags')) {
      return new Response(JSON.stringify({ models: [{ name: 'llama3:latest' }] }), { status: 200 });
    }
    return new Response(JSON.stringify({ message: { content: 'Ollama response' } }), { status: 200 });
  };

  const adapter = new OllamaAdapter({
    fetchFn: onlineFetch,
  });

  const health = await adapter.healthCheck();
  assert.equal(health, 'HEALTHY');

  const models = await adapter.listModels();
  assert.equal(models.length, 1);
  assert.equal(models[0].modelId, 'llama3:latest');
});
