import { test } from 'node:test';
import assert from 'node:assert/strict';
import { GeminiAdapter } from '../core/gateway/adapters/geminiAdapter.js';
import { OpenRouterAdapter } from '../core/gateway/adapters/openRouterAdapter.js';
import { NvidiaAdapter } from '../core/gateway/adapters/nvidiaAdapter.js';
import { GroqAdapter } from '../core/gateway/adapters/groqAdapter.js';
import { OpenAICompatibleAdapter } from '../core/gateway/adapters/openAICompatibleAdapter.js';

test('Provider adapters map invalid/rejected API keys to AUTH_ERROR', async () => {
  const rejectedFetch: typeof fetch = async () => new Response(JSON.stringify({ error: 'Unauthorized' }), { status: 401 });

  const gemini = new GeminiAdapter({ apiKey: 'bad_key', fetchFn: rejectedFetch });
  assert.equal(await gemini.healthCheck(), 'AUTH_ERROR');

  const openrouter = new OpenRouterAdapter({ apiKey: 'bad_key', fetchFn: rejectedFetch });
  assert.equal(await openrouter.healthCheck(), 'AUTH_ERROR');

  const groq = new GroqAdapter({ apiKey: 'bad_key', fetchFn: rejectedFetch });
  assert.equal(await groq.healthCheck(), 'AUTH_ERROR');
});

test('Provider adapters map unreachable/bad base URLs to OFFLINE', async () => {
  const offlineFetch: typeof fetch = async () => {
    throw new Error('fetch failed (ENOTFOUND bad.host.local)');
  };

  const custom = new OpenAICompatibleAdapter({ baseUrl: 'https://bad.host.local/v1', fetchFn: offlineFetch });
  assert.equal(await custom.healthCheck(), 'OFFLINE');

  const models = await custom.listModels();
  assert.equal(models.length, 0);
});

test('GeminiAdapter formats requests and parses responses correctly', async () => {
  const mockFetch: typeof fetch = async (url, init) => {
    assert.ok(String(url).includes('generateContent'));
    const body = JSON.parse(String(init?.body));
    assert.equal(body.contents[0].parts[0].text, 'Hello Gemini');

    return new Response(
      JSON.stringify({
        candidates: [{ content: { parts: [{ text: 'Gemini reply' }] } }],
      }),
      { status: 200 }
    );
  };

  const gemini = new GeminiAdapter({ apiKey: 'test_key', fetchFn: mockFetch });
  const res = await gemini.chat({
    providerId: 'gemini',
    modelId: 'gemini-2.0-flash',
    messages: [{ role: 'user', content: 'Hello Gemini' }],
  });

  assert.equal(res.message.content, 'Gemini reply');
});

test('OpenAICompatibleAdapter works with custom ngrok-tunneled endpoints', async () => {
  const mockFetch: typeof fetch = async (url) => {
    if (String(url).endsWith('/models')) {
      return new Response(JSON.stringify({ data: [{ id: 'custom-coder-v1' }] }), { status: 200 });
    }
    return new Response(JSON.stringify({ choices: [{ message: { content: 'Custom coder output' } }] }), { status: 200 });
  };

  // ngrok URL as custom OpenAI-compatible endpoint
  const ngrokAdapter = new OpenAICompatibleAdapter({
    id: 'custom-ngrok',
    name: 'Custom ngrok tunnel',
    baseUrl: 'https://abc-123.ngrok-free.app/v1',
    fetchFn: mockFetch,
  });

  assert.equal(await ngrokAdapter.healthCheck(), 'HEALTHY');
  const models = await ngrokAdapter.listModels();
  assert.equal(models.length, 1);
  assert.equal(models[0].modelId, 'custom-coder-v1');

  const chatResp = await ngrokAdapter.chat({
    providerId: 'custom-ngrok',
    modelId: 'custom-coder-v1',
    messages: [{ role: 'user', content: 'Run code' }],
  });
  assert.equal(chatResp.message.content, 'Custom coder output');
});
