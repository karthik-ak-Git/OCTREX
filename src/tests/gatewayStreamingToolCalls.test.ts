import { test } from 'node:test';
import assert from 'node:assert/strict';
import { UniversalModelGateway } from '../core/gateway/universalGateway.js';
import { StreamingToolMockAdapter } from '../core/gateway/streamingToolMockAdapter.js';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';
import { CancellationToken } from '../core/cancellation/cancellationToken.js';

test('Universal Gateway aggregates fragmented streaming tool call chunks into validated ToolCall', async () => {
  const gateway = new UniversalModelGateway();
  const toolAdapter = new StreamingToolMockAdapter();
  gateway.registerAdapter(toolAdapter);

  const events = [];
  for await (const event of gateway.stream({
    providerId: 'streaming-tool-mock',
    modelId: 'tool-mock-model',
    messages: [{ role: 'user', content: 'Patch index.ts' }],
  })) {
    events.push(event);
  }

  const completeEvent = events.find((e) => e.type === 'tool_call_complete');
  assert.ok(completeEvent, 'Must emit tool_call_complete event');
  assert.ok(completeEvent.toolCalls);
  assert.equal(completeEvent.toolCalls.length, 1);

  const call = completeEvent.toolCalls[0];
  assert.equal(call.id, 'call_1');
  assert.equal(call.name, 'file_patch');
  assert.equal(call.arguments.filePath, 'src/index.ts');
  assert.equal(call.arguments.startLine, 1);
  assert.equal(call.arguments.endLine, 5);
  assert.equal(call.arguments.replacementContent, 'export * from "./app";');
});

test('Universal Gateway handles multiple concurrent streamed tool calls', async () => {
  const gateway = new UniversalModelGateway();
  const multiToolAdapter = new StreamingToolMockAdapter([
    {
      type: 'tool_call_delta',
      toolCallDelta: { index: 0, id: 'call_1', name: 'file_read', argumentsDelta: '{"filePath": "app.ts"}' },
    },
    {
      type: 'tool_call_delta',
      toolCallDelta: { index: 1, id: 'call_2', name: 'terminal_execute', argumentsDelta: '{"command": "npm test"}' },
    },
    {
      type: 'finish',
      finishReason: 'tool_calls',
    },
  ]);
  gateway.registerAdapter(multiToolAdapter);

  const events = [];
  for await (const event of gateway.stream({
    providerId: 'streaming-tool-mock',
    modelId: 'tool-mock-model',
    messages: [{ role: 'user', content: 'Run test and read' }],
  })) {
    events.push(event);
  }

  const completeEvent = events.find((e) => e.type === 'tool_call_complete');
  assert.ok(completeEvent);
  assert.equal(completeEvent.toolCalls?.length, 2);
  assert.equal(completeEvent.toolCalls?.[0].name, 'file_read');
  assert.equal(completeEvent.toolCalls?.[1].name, 'terminal_execute');
  assert.equal(completeEvent.toolCalls?.[1].arguments.command, 'npm test');
});

test('Universal Gateway detects malformed tool JSON and emits MALFORMED_STREAM error', async () => {
  const gateway = new UniversalModelGateway();
  const malformedAdapter = new StreamingToolMockAdapter([
    {
      type: 'tool_call_delta',
      toolCallDelta: { index: 0, id: 'call_broken', name: 'file_patch', argumentsDelta: '{bad_json_string: missing_quotes' },
    },
    {
      type: 'finish',
      finishReason: 'tool_calls',
    },
  ]);
  gateway.registerAdapter(malformedAdapter);

  const events = [];
  for await (const event of gateway.stream({
    providerId: 'streaming-tool-mock',
    modelId: 'tool-mock-model',
    messages: [{ role: 'user', content: 'Broken call' }],
  })) {
    events.push(event);
  }

  const errEvent = events.find((e) => e.type === 'error');
  assert.ok(errEvent, 'Must emit error on malformed JSON tool stream');
  assert.equal(errEvent.error?.code, 'MALFORMED_STREAM');
});

test('Universal Gateway stream aborts cleanly on CancellationToken cancellation', async () => {
  const gateway = new UniversalModelGateway();
  const adapter = new MockProviderAdapter();
  gateway.registerAdapter(adapter);

  const token = new CancellationToken();
  token.cancel('User aborted request');

  const events = [];
  for await (const event of gateway.stream(
    {
      providerId: 'mock-provider',
      modelId: 'mock-model-v1',
      messages: [{ role: 'user', content: 'Cancel me' }],
    },
    token
  )) {
    events.push(event);
  }

  assert.equal(events.length, 1);
  assert.equal(events[0].type, 'error');
  assert.ok(events[0].error?.message.includes('User aborted request'));
});

test('Universal Gateway lists all models and checks health across all providers', async () => {
  const gateway = new UniversalModelGateway();
  gateway.registerAdapter(new MockProviderAdapter());
  gateway.registerAdapter(new StreamingToolMockAdapter());

  const healthMap = await gateway.checkAllHealth();
  assert.equal(healthMap.get('mock-provider'), 'HEALTHY');
  assert.equal(healthMap.get('streaming-tool-mock'), 'HEALTHY');

  const allModels = await gateway.listAllModels();
  assert.equal(allModels.length, 2);
  assert.ok(allModels.some((m) => m.modelId === 'mock-model-v1'));
  assert.ok(allModels.some((m) => m.modelId === 'tool-mock-model'));
});
