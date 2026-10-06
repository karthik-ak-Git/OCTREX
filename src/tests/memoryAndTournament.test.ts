import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ProjectMemoryStore } from '../core/memory/projectMemory.js';
import { SessionRecoveryManager } from '../core/recovery/sessionRecovery.js';
import { TaskStore } from '../core/task/taskStore.js';
import { TournamentEngine, CandidateSolution } from '../core/orchestration/tournamentEngine.js';

test('ProjectMemoryStore manages and persists project-specific facts', () => {
  const memStore = new ProjectMemoryStore(process.cwd());
  memStore.addConvention('Always write unit tests for new gateway adapters');
  memStore.addKnownIssue('Ollama port 11434 startup delay');

  const mem = memStore.getMemory();
  assert.ok(mem.conventions.includes('Always write unit tests for new gateway adapters'));
  assert.ok(mem.knownIssues.includes('Ollama port 11434 startup delay'));
});

test('SessionRecoveryManager safely marks in-flight active tasks as FAILED on crash recovery', () => {
  const store = new TaskStore();
  const task1 = store.createTask('Task In Flight', process.cwd());
  store.updateTaskStatus(task1.id, 'IMPLEMENTING', 'CODER');

  const task2 = store.createTask('Task Already Verified', process.cwd());
  store.updateTaskStatus(task2.id, 'VERIFIED');

  const recovery = new SessionRecoveryManager(store);
  const result = recovery.recoverSession();

  assert.equal(result.recoveredTasksCount, 2);
  assert.equal(result.activeTasksResetToFailed, 1);

  assert.equal(store.getTask(task1.id)?.status, 'FAILED');
  assert.equal(store.getTask(task2.id)?.status, 'VERIFIED');
});

test('TournamentEngine evaluates competing candidates and selects verified winner', async () => {
  const candidates: CandidateSolution[] = [
    {
      coderId: 'coder-alpha',
      solutionName: 'Alpha Solution',
      patchSummary: 'Patch with verified checks',
      workspaceOrWorktreePath: process.cwd(),
    },
  ];

  const evalResult = await TournamentEngine.evaluateCandidates('task-tourn-1', candidates);
  assert.equal(evalResult.winnerCoderId, 'coder-alpha');
  assert.equal(evalResult.winnerSolutionName, 'Alpha Solution');
  assert.ok(evalResult.reportCard.includes('Candidates Evaluated: 1'));
});
