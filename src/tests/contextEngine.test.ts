import { test } from 'node:test';
import assert from 'node:assert/strict';
import { RepoIndexer } from '../core/context/repoIndexer.js';
import { ContextEngine } from '../core/context/contextEngine.js';

test('RepoIndexer scans workspace and discovers code symbols and related tests', async () => {
  const indexer = new RepoIndexer(process.cwd());
  const count = await indexer.scanWorkspace();
  assert.ok(count > 0, 'Workspace files must be indexed');

  // Search for known symbols in OCTREX
  const symbols = indexer.findSymbols('UniversalModelGateway');
  assert.ok(symbols.length > 0, 'Must find UniversalModelGateway symbol');
  assert.equal(symbols[0].symbolName, 'UniversalModelGateway');

  // Find related test files
  const relatedTests = indexer.findRelatedTests('src/core/gateway/universalGateway.ts');
  assert.ok(relatedTests.length > 0, 'Must find at least one related test file');
  assert.ok(relatedTests.some((t) => t.includes('gatewayStreamingToolCalls.test.ts') || t.includes('test')));
});

test('ContextEngine prunes large tool outputs retaining critical head and tail', () => {
  const engine = new ContextEngine();
  const largeLogs = Array.from({ length: 300 }, (_, i) => `Log line ${i + 1}`).join('\n');

  const pruned = engine.pruneToolOutput(largeLogs, 50);
  assert.ok(pruned.includes('Log line 1'));
  assert.ok(pruned.includes('Log line 300'));
  assert.ok(pruned.includes('truncated to conserve context'));
});

test('ContextEngine compacts long conversations while preserving system rules', () => {
  const engine = new ContextEngine();
  const chat = [
    { role: 'system' as const, content: 'You are OCTREX' },
    ...Array.from({ length: 12 }, (_, i) => ({
      role: (i % 2 === 0 ? 'user' : 'assistant') as 'user' | 'assistant',
      content: `Turn content ${i + 1}`,
    })),
  ];

  const compacted = engine.compactConversation(chat, 4);
  assert.ok(compacted.length < chat.length);
  assert.equal(compacted[0].content, 'You are OCTREX');
  assert.ok(compacted.some((m) => m.content.includes('Previous Context Summary')));
});

test('ContextEngine assembles structured budgeted prompt with memory and snippets', () => {
  const engine = new ContextEngine();
  const prompt = engine.assemblePrompt({
    systemInstruction: 'Act as CODER agent',
    taskPrompt: 'Fix bug in gateway',
    memory: {
      framework: 'TypeScript Node ESM',
      buildCommands: ['npm run build'],
      testCommands: ['npm test'],
      conventions: ['Strict types', 'Async/await'],
      knownIssues: [],
    },
    snippets: [
      {
        filePath: 'src/core/gateway.ts',
        startLine: 1,
        endLine: 20,
        content: 'class Gateway {}',
        reason: 'Target file',
      },
    ],
  });

  assert.ok(prompt.some((m) => m.content.includes('Act as CODER agent')));
  assert.ok(prompt.some((m) => m.content.includes('npm run build')));
  assert.ok(prompt.some((m) => m.content.includes('class Gateway {}')));
  assert.equal(prompt[prompt.length - 1].content, 'Fix bug in gateway');
});
