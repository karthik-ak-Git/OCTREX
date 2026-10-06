import { test } from 'node:test';
import assert from 'node:assert/strict';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';
test('MockProviderAdapter implements normalized interface contracts', async () => {
    const adapter = new MockProviderAdapter();
    assert.equal(adapter.id, 'mock-provider');
    assert.equal(await adapter.healthCheck(), 'HEALTHY');
    const models = await adapter.listModels();
    assert.equal(models.length, 1);
    assert.equal(models[0].modelId, 'mock-model-v1');
    const chatResp = await adapter.chat({
        providerId: adapter.id,
        modelId: 'mock-model-v1',
        messages: [{ role: 'user', content: 'Hello V4' }],
    });
    assert.equal(chatResp.finishReason, 'stop');
    assert.ok(chatResp.message.content.includes('Hello V4'));
    const streamChunks = [];
    for await (const chunk of adapter.stream({
        providerId: adapter.id,
        modelId: 'mock-model-v1',
        messages: [{ role: 'user', content: 'Streaming test' }],
    })) {
        if (chunk.type === 'text_delta' && chunk.delta) {
            streamChunks.push(chunk.delta);
        }
    }
    assert.ok(streamChunks.length > 0);
    assert.ok(streamChunks.join('').includes('Streaming test'));
});
//# sourceMappingURL=contracts.test.js.map