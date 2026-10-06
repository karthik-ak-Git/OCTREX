import test from 'node:test';
import assert from 'node:assert/strict';
import { CraxGptAdapter, CRAX_GPT_DEFAULT_MODELS } from '../core/gateway/adapters/craxGptAdapter.js';
import { UniversalModelGateway } from '../core/gateway/universalGateway.js';
import { ModelRouter } from '../core/router/modelRouter.js';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';
import { CloudConsentGuard } from '../core/security/cloudConsentGuard.js';
import { ProviderLogoService } from '../core/gateway/providerLogos.js';
import { UIEventEmitter } from '../core/events/uiEventEmitter.js';

test('CraxGptAdapter initializes with official defaults', () => {
  const adapter = new CraxGptAdapter();
  assert.equal(adapter.id, 'crax-gpt');
  assert.equal(adapter.name, 'crax-gpt');
  assert.equal(adapter.getBaseUrl(), 'https://gpt.crax.lol/v1');
  assert.equal(adapter.getSetupUrl(), 'https://gpt.crax.lol');
});

test('CraxGptAdapter returns AUTH_ERROR when API key is missing', async () => {
  const adapter = new CraxGptAdapter({ apiKey: undefined });
  const health = await adapter.healthCheck();
  assert.equal(health, 'AUTH_ERROR');
});

test('CraxGptAdapter healthCheck maps HTTP responses accurately', async () => {
  // 1. HEALTHY on 200
  const healthyAdapter = new CraxGptAdapter({
    apiKey: 'test-valid-key',
    fetchFn: async () => new Response(JSON.stringify({ data: [] }), { status: 200 }),
  });
  assert.equal(await healthyAdapter.healthCheck(), 'HEALTHY');

  // 2. AUTH_ERROR on 401
  const authErrAdapter = new CraxGptAdapter({
    apiKey: 'test-invalid-key',
    fetchFn: async () => new Response('Unauthorized', { status: 401 }),
  });
  assert.equal(await authErrAdapter.healthCheck(), 'AUTH_ERROR');

  // 3. RATE_LIMITED on 429
  const rateLimitAdapter = new CraxGptAdapter({
    apiKey: 'test-key',
    fetchFn: async () => new Response('Rate limited', { status: 429 }),
  });
  assert.equal(await rateLimitAdapter.healthCheck(), 'RATE_LIMITED');

  // 4. OFFLINE on network error
  const offlineAdapter = new CraxGptAdapter({
    apiKey: 'test-key',
    fetchFn: async () => {
      throw new Error('fetch failed');
    },
  });
  assert.equal(await offlineAdapter.healthCheck(), 'OFFLINE');
});

test('CraxGptAdapter validates credentials accurately', async () => {
  const adapter = new CraxGptAdapter({
    fetchFn: async (_url, init) => {
      const auth = (init?.headers as Record<string, string>)?.[`Authorization`];
      if (auth === 'Bearer valid-secret-key') {
        return new Response(JSON.stringify({ data: [] }), { status: 200 });
      }
      return new Response('Invalid key', { status: 401 });
    },
  });

  const valid = await adapter.validateCredentials({ apiKey: 'valid-secret-key' });
  const invalid = await adapter.validateCredentials({ apiKey: 'wrong-key' });
  const missing = await adapter.validateCredentials({});

  assert.equal(valid, true);
  assert.equal(invalid, false);
  assert.equal(missing, false);
});

test('CraxGptAdapter dynamically discovers models via /v1/models', async () => {
  const dynamicModels = [
    { id: 'glm-5.3', name: 'GLM-5.3 Enterprise', context_length: 131072 },
    { id: 'custom-coder-v1', name: 'Custom Coder', context_length: 65536 },
    { id: 'vision-ultra-4k', name: 'Vision Ultra', context_length: 32768 },
  ];

  let requestHeaders: Record<string, string> | undefined;

  const adapter = new CraxGptAdapter({
    apiKey: 'live-key-123',
    fetchFn: async (url, init) => {
      requestHeaders = init?.headers as Record<string, string>;
      return new Response(JSON.stringify({ data: dynamicModels }), { status: 200 });
    },
  });

  const models = await adapter.refreshModels();
  assert.equal(models.length, 3);
  assert.equal(models[0].modelId, 'glm-5.3');
  assert.equal(models[0].capabilities.supportsReasoning, true);
  assert.equal(models[0].capabilities.supportsVision, true);
  assert.equal(models[0].capabilities.isLocal, false);

  assert.equal(models[1].modelId, 'custom-coder-v1');
  assert.equal(models[2].modelId, 'vision-ultra-4k');
  assert.equal(models[2].capabilities.supportsVision, true);

  // Verify custom branding headers
  assert.equal(requestHeaders?.['Authorization'], 'Bearer live-key-123');
  assert.equal(requestHeaders?.['HTTP-Referer'], 'https://github.com/karthik-ak-Git/OCTREX');
  assert.equal(requestHeaders?.['X-Title'], 'OCTREX CODE V4');
});

test('CraxGptAdapter refreshModels updates live catalog when remote models change', async () => {
  let catalogVersion = 1;

  const adapter = new CraxGptAdapter({
    apiKey: 'live-key-123',
    fetchFn: async () => {
      if (catalogVersion === 1) {
        return new Response(JSON.stringify({ data: [{ id: 'glm-5.3' }] }), { status: 200 });
      } else {
        return new Response(
          JSON.stringify({
            data: [{ id: 'glm-5.3' }, { id: 'glm-6.0-preview' }, { id: 'deepseek-v3' }],
          }),
          { status: 200 }
        );
      }
    },
  });

  const initialList = await adapter.refreshModels();
  assert.equal(initialList.length, 1);
  assert.equal(initialList[0].modelId, 'glm-5.3');

  // Trigger catalog change on remote
  catalogVersion = 2;
  const updatedList = await adapter.refreshModels();
  assert.equal(updatedList.length, 3);
  assert.equal(updatedList[1].modelId, 'glm-6.0-preview');
  assert.equal(updatedList[2].modelId, 'deepseek-v3');
});

test('CraxGptAdapter falls back gracefully to default catalog when offline', async () => {
  const offlineAdapter = new CraxGptAdapter({
    apiKey: 'some-key',
    fetchFn: async () => {
      throw new Error('network timeout');
    },
  });

  const models = await offlineAdapter.listModels();
  assert.equal(models.length, CRAX_GPT_DEFAULT_MODELS.length);
  assert.equal(models[0].modelId, 'glm-5.3');
  assert.equal(models[0].health, 'OFFLINE');
});

test('CraxGptAdapter formats chat request and parses response', async () => {
  let capturedBody: any;

  const adapter = new CraxGptAdapter({
    apiKey: 'test-chat-key',
    fetchFn: async (url, init) => {
      capturedBody = JSON.parse(init?.body as string);
      return new Response(
        JSON.stringify({
          id: 'crax-chat-999',
          choices: [
            {
              message: {
                role: 'assistant',
                content: 'Refactored successfully.',
                tool_calls: [
                  {
                    id: 'call_1',
                    function: { name: 'patch_file', arguments: JSON.stringify({ path: 'src/main.ts' }) },
                  },
                ],
              },
              finish_reason: 'tool_calls',
            },
          ],
          usage: { prompt_tokens: 15, completion_tokens: 25, total_tokens: 40 },
        }),
        { status: 200 }
      );
    },
  });

  const response = await adapter.chat({
    providerId: 'crax-gpt',
    modelId: 'glm-5.3',
    messages: [{ role: 'user', content: 'Refactor code' }],
    tools: [
      {
        name: 'patch_file',
        description: 'Patch a file',
        parameters: { type: 'object', properties: { path: { type: 'string' } } },
      },
    ],
  });

  assert.equal(capturedBody.model, 'glm-5.3');
  assert.equal(capturedBody.tools.length, 1);
  assert.equal(capturedBody.tools[0].function.name, 'patch_file');

  assert.equal(response.id, 'crax-chat-999');
  assert.equal(response.message.content, 'Refactored successfully.');
  assert.equal(response.message.toolCalls?.length, 1);
  assert.equal(response.message.toolCalls?.[0].name, 'patch_file');
  assert.equal(response.message.toolCalls?.[0].arguments.path, 'src/main.ts');
  assert.equal(response.finishReason, 'tool_calls');
  assert.equal(response.usage?.totalTokens, 40);
});

test('CraxGptAdapter streams tokens and completes with finish chunk', async () => {
  const sseData = [
    'data: {"choices":[{"delta":{"content":"Hello"}}]}\n\n',
    'data: {"choices":[{"delta":{"content":" World"}}]}\n\n',
    'data: [DONE]\n\n',
  ].join('');

  const adapter = new CraxGptAdapter({
    apiKey: 'test-stream-key',
    fetchFn: async () => {
      const stream = new ReadableStream({
        start(controller) {
          controller.enqueue(new TextEncoder().encode(sseData));
          controller.close();
        },
      });
      return new Response(stream, { status: 200 });
    },
  });

  const chunks: string[] = [];
  for await (const chunk of adapter.stream({
    providerId: 'crax-gpt',
    modelId: 'glm-5.3',
    messages: [{ role: 'user', content: 'Hi' }],
  })) {
    if (chunk.type === 'text_delta' && chunk.delta) {
      chunks.push(chunk.delta);
    }
  }

  assert.equal(chunks.join(''), 'Hello World');
});

test('CraxGptAdapter parses 429 Rate Limits with Retry-After header', async () => {
  const adapter = new CraxGptAdapter({
    apiKey: 'test-key',
    fetchFn: async () => {
      return new Response(JSON.stringify({ error: { message: 'Rate limit exceeded' } }), {
        status: 429,
        headers: { 'retry-after': '45' },
      });
    },
  });

  await assert.rejects(
    async () => {
      await adapter.chat({
        providerId: 'crax-gpt',
        modelId: 'glm-5.3',
        messages: [{ role: 'user', content: 'Test' }],
      });
    },
    (err: any) => {
      assert.equal(err.code, 'RATE_LIMITED');
      assert.equal(err.retryable, true);
      assert.ok(err.message.includes('45s'));
      return true;
    }
  );
});

test('CraxGptAdapter maps 404 to MODEL_NOT_FOUND and 500 to PROVIDER_UNAVAILABLE', async () => {
  const notFoundAdapter = new CraxGptAdapter({
    apiKey: 'key',
    fetchFn: async () => new Response('Model does not exist', { status: 404 }),
  });

  await assert.rejects(
    async () => {
      await notFoundAdapter.chat({
        providerId: 'crax-gpt',
        modelId: 'non-existent-model',
        messages: [{ role: 'user', content: 'Test' }],
      });
    },
    (err: any) => {
      assert.equal(err.code, 'MODEL_NOT_FOUND');
      assert.equal(err.retryable, false);
      return true;
    }
  );

  const serverErrorAdapter = new CraxGptAdapter({
    apiKey: 'key',
    fetchFn: async () => new Response('Internal Server Error', { status: 500 }),
  });

  await assert.rejects(
    async () => {
      await serverErrorAdapter.chat({
        providerId: 'crax-gpt',
        modelId: 'glm-5.3',
        messages: [{ role: 'user', content: 'Test' }],
      });
    },
    (err: any) => {
      assert.equal(err.code, 'PROVIDER_UNAVAILABLE');
      assert.equal(err.retryable, true);
      return true;
    }
  );
});

test('CloudConsentGuard gates crax-gpt requests when cloud-code consent is missing (M4)', async () => {
  const workspacePath = 'D:\\OCTREX\\test-consent-crax';
  CloudConsentGuard.revokeConsent(workspacePath);

  const gateway = new UniversalModelGateway();
  const craxAdapter = new CraxGptAdapter({
    apiKey: 'mock-key',
    fetchFn: async () =>
      new Response(JSON.stringify({ choices: [{ message: { content: 'Secret code analyzed' } }] }), { status: 200 }),
  });

  gateway.registerAdapter(craxAdapter);

  // 1. Without consent: Blocked
  await assert.rejects(
    async () => {
      await gateway.chat(
        {
          providerId: 'crax-gpt',
          modelId: 'glm-5.3',
          messages: [{ role: 'user', content: '```typescript\nconst apiSecret = 123;\n```' }],
        },
        undefined,
        workspacePath
      );
    },
    (err: any) => {
      assert.equal(err.code, 'PERMISSION_DENIED');
      assert.ok(err.message.includes('Cloud-Code Consent'));
      return true;
    }
  );

  // 2. Grant consent: Allowed
  CloudConsentGuard.grantConsent(workspacePath);
  const response = await gateway.chat(
    {
      providerId: 'crax-gpt',
      modelId: 'glm-5.3',
      messages: [{ role: 'user', content: '```typescript\nconst apiSecret = 123;\n```' }],
    },
    undefined,
    workspacePath
  );
  assert.equal(response.message.content, 'Secret code analyzed');
});

test('ModelRouter selects crax-gpt in POWERFUL mode and falls back on 429 failure', async () => {
  const workspace = 'D:\\OCTREX\\router-crax-test';
  CloudConsentGuard.grantConsent(workspace);

  const gateway = new UniversalModelGateway();
  const events = new UIEventEmitter();

  let craxHitCount = 0;
  const failingCraxAdapter = new CraxGptAdapter({
    apiKey: 'crax-key',
    fetchFn: async (url) => {
      if (url.toString().includes('/models')) {
        return new Response(JSON.stringify({ data: [{ id: 'glm-5.3' }] }), { status: 200 });
      }
      craxHitCount++;
      return new Response('Rate limited', { status: 429, headers: { 'retry-after': '30' } });
    },
  });

  const fallbackMock = new MockProviderAdapter({
    delayMs: 1,
  });

  gateway.registerAdapter(failingCraxAdapter);
  gateway.registerAdapter(fallbackMock);

  const router = new ModelRouter(gateway, events);
  router.setDefaultStrategy('POWERFUL');

  const fallbackEvents: any[] = [];
  events.subscribe('*', (ev) => {
    if (ev.eventType === 'model_fallback_occurred') {
      fallbackEvents.push(ev.data);
    }
  });

  const response = await router.executeChatWithFallback(
    {
      taskId: 'task-crax-fallback',
      workspacePath: workspace,
      strategyMode: 'POWERFUL',
    },
    [{ role: 'user', content: 'Execute reasoning task' }]
  );

  assert.equal(craxHitCount, 1);
  assert.equal(response.providerId, 'mock-provider');
  assert.ok(fallbackEvents.some((e) => e.action === 'fallback.started' && e.failedProvider === 'crax-gpt'));
  assert.ok(fallbackEvents.some((e) => e.action === 'fallback.completed' && e.newProvider === 'mock-provider'));
});

test('ProviderLogoService returns valid SVGs and formats active route badges', () => {
  const craxMeta = ProviderLogoService.getMetadata('crax-gpt');
  assert.equal(craxMeta.id, 'crax-gpt');
  assert.equal(craxMeta.name, 'crax-gpt');
  assert.equal(craxMeta.setupUrl, 'https://gpt.crax.lol');
  assert.ok(craxMeta.iconSvg.includes('<svg'));

  const geminiMeta = ProviderLogoService.getMetadata('gemini');
  assert.ok(geminiMeta.iconSvg.includes('<svg'));

  const badge = ProviderLogoService.formatRouteBadge('crax-gpt', 'glm-5.3');
  assert.equal(badge.textBadge, '[crax-gpt] crax-gpt • glm-5.3');
  assert.ok(badge.htmlBadge.includes('octrex-route-badge'));
  assert.ok(badge.htmlBadge.includes('glm-5.3'));
});
