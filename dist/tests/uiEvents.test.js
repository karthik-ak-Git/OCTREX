import { test } from 'node:test';
import assert from 'node:assert/strict';
import { UIEventEmitter } from '../core/events/uiEventEmitter.js';
test('UIEventEmitter emits structured, schema-compliant events for frontend integration', () => {
    const emitter = new UIEventEmitter();
    const receivedEvents = [];
    emitter.subscribe('agent_started', (payload) => {
        receivedEvents.push(payload);
    });
    const payload = emitter.emit('task-ui-202', 'agent_started', {
        agentRole: 'PLANNER',
        modelId: 'gemini-1.5-pro',
    });
    assert.equal(receivedEvents.length, 1);
    assert.equal(receivedEvents[0].taskId, 'task-ui-202');
    assert.equal(receivedEvents[0].eventType, 'agent_started');
    assert.equal(receivedEvents[0].data.agentRole, 'PLANNER');
    assert.ok(payload.timestamp);
});
test('UIEventEmitter supports wildcard subscribers for full UI state sync', () => {
    const emitter = new UIEventEmitter();
    const allEvents = [];
    emitter.subscribe('*', (payload) => {
        allEvents.push(payload.eventType);
    });
    emitter.emit('t-1', 'task_updated', { status: 'PLANNING' });
    emitter.emit('t-1', 'diff_generated', { file: 'index.ts' });
    emitter.emit('t-1', 'verification_completed', { status: 'VERIFIED' });
    assert.deepEqual(allEvents, ['task_updated', 'diff_generated', 'verification_completed']);
});
//# sourceMappingURL=uiEvents.test.js.map