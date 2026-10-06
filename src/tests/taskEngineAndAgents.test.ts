import { test } from 'node:test';
import assert from 'node:assert/strict';
import { TaskStore } from '../core/task/taskStore.js';
import { AgentOrchestrator } from '../core/agents/agentOrchestrator.js';
import { ModelRouter } from '../core/router/modelRouter.js';
import { UniversalModelGateway } from '../core/gateway/universalGateway.js';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';
import { UIEventEmitter } from '../core/events/uiEventEmitter.js';
import { CancellationToken } from '../core/cancellation/cancellationToken.js';
import { CloudConsentGuard } from '../core/security/cloudConsentGuard.js';

test('TaskStore manages task state with standardized task IDs', () => {
  const store = new TaskStore();
  const task = store.createTask('Implement auth bugfix', process.cwd());

  assert.ok(task.id.startsWith('task_'));
  assert.equal(task.status, 'RECEIVED');

  store.updateTaskStatus(task.id, 'PLANNING', 'PLANNER');
  const updated = store.getTask(task.id);
  assert.equal(updated?.status, 'PLANNING');
  assert.equal(updated?.activeAgent, 'PLANNER');
});

test('AgentOrchestrator executes multi-agent lifecycle and verifies results', async () => {
  CloudConsentGuard.grantConsent(process.cwd());

  const taskStore = new TaskStore();
  const gateway = new UniversalModelGateway();
  gateway.registerAdapter(new MockProviderAdapter());

  const emitter = new UIEventEmitter();
  const stateTransitions: string[] = [];

  emitter.subscribe('task_updated', (payload) => {
    stateTransitions.push(payload.data.status);
  });

  const router = new ModelRouter(gateway, emitter);
  const orchestrator = new AgentOrchestrator(taskStore, router, emitter);

  const result = await orchestrator.executeTask('Refactor index.ts exports', process.cwd());

  assert.ok(result.taskId.startsWith('task_'));
  assert.equal(result.status, 'VERIFIED');
  assert.ok(result.plan);
  assert.ok(result.reviewNotes);

  assert.ok(stateTransitions.includes('RECEIVED'));
  assert.ok(stateTransitions.includes('REPOSITORY_ANALYSIS'));
  assert.ok(stateTransitions.includes('PLANNING'));
  assert.ok(stateTransitions.includes('IMPLEMENTING'));
  assert.ok(stateTransitions.includes('TESTING'));
  assert.ok(stateTransitions.includes('REVIEWING'));
  assert.ok(stateTransitions.includes('VERIFIED'));
});

test('AgentOrchestrator stops execution cleanly when task is cancelled', async () => {
  const taskStore = new TaskStore();
  const gateway = new UniversalModelGateway();
  gateway.registerAdapter(new MockProviderAdapter());

  const emitter = new UIEventEmitter();
  const router = new ModelRouter(gateway, emitter);
  const orchestrator = new AgentOrchestrator(taskStore, router, emitter);

  const token = new CancellationToken();
  token.cancel('User aborted task mid-flight');

  const result = await orchestrator.executeTask('Add feature', process.cwd(), token);
  assert.equal(result.status, 'CANCELLED');
});
