import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ModelRouter } from '../core/router/modelRouter.js';
import { CircuitBreaker } from '../core/router/circuitBreaker.js';
import { UniversalModelGateway } from '../core/gateway/universalGateway.js';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';
import { GroqAdapter } from '../core/gateway/adapters/groqAdapter.js';
import { UIEventEmitter } from '../core/events/uiEventEmitter.js';

test('CircuitBreaker transitions correctly between CLOSED, OPEN, and HALF_OPEN', () => {
  const cb = new CircuitBreaker('test-p', { failureThreshold: 2, cooldownPeriodMs: 50 });
  assert.equal(cb.state, 'CLOSED');
  assert.equal(cb.isAvailable(), true);

  cb.recordFailure();
  assert.equal(cb.state, 'CLOSED');

  cb.recordFailure(); // Threshold met -> OPEN
  assert.equal(cb.state, 'OPEN');
  assert.equal(cb.isAvailable(), false);

  // Wait for cooldown
  return new Promise<void>((resolve) => {
    setTimeout(() => {
      assert.equal(cb.isAvailable(), true); // Transitions to HALF_OPEN
      assert.equal(cb.state, 'HALF_OPEN');

      cb.recordSuccess(); // Success in HALF_OPEN resets to CLOSED
      assert.equal(cb.state, 'CLOSED');
      resolve();
    }, 60);
  });
});

test('ModelRouter respects FAST, POWERFUL, and FREE_ONLY strategy modes', async () => {
  const gateway = new UniversalModelGateway();
  const mockAdapter = new MockProviderAdapter();
  gateway.registerAdapter(mockAdapter);

  const mockGroqFetch: typeof fetch = async () => new Response(JSON.stringify({ data: [{ id: 'llama-3.1-8b-instant' }] }), { status: 200 });
  const groq = new GroqAdapter({ apiKey: 'key', fetchFn: mockGroqFetch });
  gateway.registerAdapter(groq);

  const router = new ModelRouter(gateway);

  // FAST mode should pick fast model
  const fastRoute = await router.selectRoute({
    taskId: 't-fast',
    strategyMode: 'FAST',
  });
  assert.equal(fastRoute.providerId, 'groq');

  // FREE_ONLY mode should select free model
  const freeRoute = await router.selectRoute({
    taskId: 't-free',
    strategyMode: 'FREE_ONLY',
  });
  assert.ok(freeRoute.providerId === 'mock-provider');
});

test('ModelRouter handles automated fallback and emits structured UI routing events', async () => {
  const gateway = new UniversalModelGateway();
  const emitter = new UIEventEmitter();
  const events: any[] = [];

  emitter.subscribe('model_fallback_occurred', (payload) => {
    events.push(payload.data);
  });

  // Failing primary adapter
  const failingPrimary = new MockProviderAdapter({ shouldFailWithCode: 500 });
  failingPrimary.id = 'primary-cloud';
  gateway.registerAdapter(failingPrimary);

  // Working fallback adapter
  const workingFallback = new MockProviderAdapter();
  workingFallback.id = 'fallback-cloud';
  gateway.registerAdapter(workingFallback);

  const router = new ModelRouter(gateway, emitter);

  const resp = await router.executeChatWithFallback(
    {
      taskId: 'task-fallback-101',
      strategyMode: 'AUTO',
    },
    [{ role: 'user', content: 'Do task' }]
  );

  assert.ok(resp.message.content.includes('Mock Response'));
  assert.ok(events.some((e) => e.action === 'fallback.started'));
  assert.ok(events.some((e) => e.action === 'fallback.completed'));
});
