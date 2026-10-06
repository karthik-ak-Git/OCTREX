import { test, beforeEach } from 'node:test';
import assert from 'node:assert/strict';
import { UniversalModelGateway } from '../core/gateway/universalGateway.js';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';
import { OllamaAdapter } from '../core/gateway/ollamaAdapter.js';
import { CloudConsentGuard } from '../core/security/cloudConsentGuard.js';
import { ModelRouter } from '../core/router/modelRouter.js';

const TEST_WORKSPACE = 'd:/OCTREX/sample-project';

beforeEach(() => {
  CloudConsentGuard.clearAll();
});

test('M4 Regression: Cloud provider + No Consent = Blocked with PERMISSION_DENIED', async () => {
  const gateway = new UniversalModelGateway();
  const cloudMock = new MockProviderAdapter(); // isLocal: false
  gateway.registerAdapter(cloudMock);

  // Request containing repository code context
  const req = {
    providerId: 'mock-provider',
    modelId: 'mock-model-v1',
    messages: [
      {
        role: 'system' as const,
        content: '[Relevant Repository Code Context]:\n--- File: src/auth.ts ---\nfunction verifyToken() {}',
      },
      {
        role: 'user' as const,
        content: 'Fix token bug',
      },
    ],
  };

  // Chat without consent must throw PERMISSION_DENIED
  await assert.rejects(
    async () => {
      await gateway.chat(req, undefined, TEST_WORKSPACE);
    },
    (err: any) => {
      return (
        err.code === 'PERMISSION_DENIED' &&
        err.message.includes('Cloud-code consent required')
      );
    }
  );

  // Stream without consent must emit PERMISSION_DENIED error event
  const streamEvents = [];
  for await (const event of gateway.stream(req, undefined, TEST_WORKSPACE)) {
    streamEvents.push(event);
  }

  assert.equal(streamEvents.length, 1);
  assert.equal(streamEvents[0].type, 'error');
  assert.equal(streamEvents[0].error?.code, 'PERMISSION_DENIED');
});

test('M4 Regression: Cloud provider + Consent = Allowed', async () => {
  const gateway = new UniversalModelGateway();
  const cloudMock = new MockProviderAdapter();
  gateway.registerAdapter(cloudMock);

  // Grant explicit backend consent for workspace
  CloudConsentGuard.grantConsent(TEST_WORKSPACE);
  assert.equal(CloudConsentGuard.hasConsent(TEST_WORKSPACE), true);

  const req = {
    providerId: 'mock-provider',
    modelId: 'mock-model-v1',
    messages: [
      {
        role: 'system' as const,
        content: '[Relevant Repository Code Context]:\n--- File: src/auth.ts ---\nfunction verifyToken() {}',
      },
      {
        role: 'user' as const,
        content: 'Fix token bug',
      },
    ],
  };

  const response = await gateway.chat(req, undefined, TEST_WORKSPACE);
  assert.ok(response.message.content);
  assert.equal(response.finishReason, 'stop');
});

test('M4 Regression: Local-only provider does not require cloud consent', async () => {
  const gateway = new UniversalModelGateway();
  const onlineFetch: typeof fetch = async (url) => {
    if (String(url).endsWith('/api/tags')) {
      return new Response(JSON.stringify({ models: [{ name: 'llama3:latest' }] }), { status: 200 });
    }
    return new Response(JSON.stringify({ message: { content: 'Local Ollama response' } }), { status: 200 });
  };

  const localOllama = new OllamaAdapter({ fetchFn: onlineFetch }); // isLocal: true
  gateway.registerAdapter(localOllama);

  // No consent granted for workspace
  assert.equal(CloudConsentGuard.hasConsent(TEST_WORKSPACE), false);

  const req = {
    providerId: 'ollama',
    modelId: 'llama3:latest',
    messages: [
      {
        role: 'system' as const,
        content: '[Relevant Repository Code Context]:\n--- File: src/auth.ts ---\nfunction verifyToken() {}',
      },
      {
        role: 'user' as const,
        content: 'Fix token bug',
      },
    ],
  };

  // Local chat must succeed without requiring cloud consent
  const response = await gateway.chat(req, undefined, TEST_WORKSPACE);
  assert.equal(response.message.content, 'Local Ollama response');
});

test('M4 Regression: Consent state cannot be bypassed through ModelRouter or fallback paths', async () => {
  const gateway = new UniversalModelGateway();
  const cloudPrimary = new MockProviderAdapter({ shouldFailWithCode: 500 });
  cloudPrimary.id = 'cloud-primary';
  gateway.registerAdapter(cloudPrimary);

  const cloudFallback = new MockProviderAdapter();
  cloudFallback.id = 'cloud-fallback';
  gateway.registerAdapter(cloudFallback);

  const router = new ModelRouter(gateway);

  // No consent
  assert.equal(CloudConsentGuard.hasConsent(TEST_WORKSPACE), false);

  const messages = [
    {
      role: 'system' as const,
      content: '[Relevant Repository Code Context]:\n--- File: src/main.ts ---\nconst a = 1;',
    },
    { role: 'user' as const, content: 'Check code' },
  ];

  await assert.rejects(
    async () => {
      await router.executeChatWithFallback(
        { taskId: 't-bypass', workspacePath: TEST_WORKSPACE, strategyMode: 'AUTO' },
        messages
      );
    },
    (err: any) => {
      return err.code === 'PERMISSION_DENIED';
    }
  );
});

test('M4 Regression: General non-code questions are permitted without cloud consent', async () => {
  const gateway = new UniversalModelGateway();
  const cloudMock = new MockProviderAdapter();
  gateway.registerAdapter(cloudMock);

  // No consent
  assert.equal(CloudConsentGuard.hasConsent(TEST_WORKSPACE), false);

  const generalReq = {
    providerId: 'mock-provider',
    modelId: 'mock-model-v1',
    messages: [
      {
        role: 'user' as const,
        content: 'What is the capital of France?',
      },
    ],
  };

  // General non-code request allowed
  const response = await gateway.chat(generalReq, undefined, TEST_WORKSPACE);
  assert.ok(response.message.content);
});
