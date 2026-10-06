import { test } from 'node:test';
import assert from 'node:assert/strict';
import { OpenCodeAdapter, OPENCODE_FREE_MODELS } from '../core/gateway/opencodeAdapter.js';

test('OpenCodeAdapter lists pre-configured free models correctly', async () => {
  const mockFetch: typeof fetch = async (url) => {
    if (String(url).endsWith('/models')) {
      return new Response(JSON.stringify({ data: [{ id: 'google/gemini-2.0-flash-exp:free' }] }), { status: 200 });
    }
    return new Response(JSON.stringify({}), { status: 200 });
  };

  const adapter = new OpenCodeAdapter({ fetchFn: mockFetch });
  const health = await adapter.healthCheck();
  assert.equal(health, 'HEALTHY');

  const models = await adapter.listModels();
  assert.ok(models.length >= OPENCODE_FREE_MODELS.length);

  const geminiFree = models.find((m) => m.modelId === 'google/gemini-2.0-flash-exp:free');
  assert.ok(geminiFree);
  assert.equal(geminiFree.capabilities.isFree, true);
  assert.equal(geminiFree.capabilities.supportsReasoning, true);
});

test('OpenCodeAdapter sends chat completion request to OpenCode gateway endpoint', async () => {
  const mockFetch: typeof fetch = async (url, init) => {
    if (String(url).endsWith('/chat/completions')) {
      const body = JSON.parse(String(init?.body));
      assert.equal(body.model, 'google/gemini-2.0-flash-exp:free');
      return new Response(
        JSON.stringify({
          id: 'gen-123',
          choices: [{ message: { content: 'OpenCode free response' }, finish_reason: 'stop' }],
          usage: { prompt_tokens: 12, completion_tokens: 8, total_tokens: 20 },
        }),
        { status: 200 }
      );
    }
    return new Response(JSON.stringify({}), { status: 200 });
  };

  const adapter = new OpenCodeAdapter({ fetchFn: mockFetch });
  const response = await adapter.chat({
    providerId: 'opencode',
    modelId: 'google/gemini-2.0-flash-exp:free',
    messages: [{ role: 'user', content: 'Generate function' }],
  });

  assert.equal(response.message.content, 'OpenCode free response');
  assert.equal(response.usage?.totalTokens, 20);
});
