import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  FrozenSessionContract,
  FrozenProjectContract,
  FrozenProviderContract,
  FrozenVerificationContract,
  FrozenDiffContract,
} from '../core/contracts/index.js';

test('Frozen backend contracts provide complete typing for frontend consumer', () => {
  const session: FrozenSessionContract = {
    sessionId: 'sess_1',
    workspacePath: process.cwd(),
    historyTasks: [],
  };
  assert.equal(session.sessionId, 'sess_1');

  const proj: FrozenProjectContract = {
    workspacePath: process.cwd(),
    buildCommands: ['npm run build'],
    testCommands: ['npm test'],
    conventions: ['Strict types'],
  };
  assert.equal(proj.buildCommands.length, 1);

  const provider: FrozenProviderContract = {
    providerId: 'gemini',
    displayName: 'Google Gemini',
    health: 'HEALTHY',
    models: [],
  };
  assert.equal(provider.health, 'HEALTHY');

  const diffContract: FrozenDiffContract = {
    taskId: 'task_1',
    diff: '+ added line',
    filesChanged: ['src/index.ts'],
  };
  assert.equal(diffContract.filesChanged.length, 1);
});
