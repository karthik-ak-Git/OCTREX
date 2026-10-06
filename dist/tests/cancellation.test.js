import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CancellationToken, TaskCancellationManager } from '../core/cancellation/cancellationToken.js';
test('CancellationToken triggers callbacks immediately on cancellation', () => {
    const token = new CancellationToken();
    let cleanUpExecuted = false;
    token.onCancelled((reason) => {
        cleanUpExecuted = true;
        assert.equal(reason, 'User cancelled operation');
    });
    assert.equal(token.isCancelled, false);
    token.cancel('User cancelled operation');
    assert.equal(token.isCancelled, true);
    assert.equal(cleanUpExecuted, true);
    assert.throws(() => token.throwIfCancelled(), /Operation cancelled: User cancelled operation/);
});
test('TaskCancellationManager handles provider disconnect cleanly to prevent zombie tasks', () => {
    const manager = new TaskCancellationManager();
    const token = manager.getOrCreateToken('task-provider-disconnect-01');
    let processTerminated = false;
    token.onCancelled((reason) => {
        processTerminated = true;
        assert.ok(reason.includes('Provider disconnect detected on provider'));
    });
    manager.handleProviderDisconnect('task-provider-disconnect-01', 'openrouter');
    assert.equal(token.isCancelled, true);
    assert.equal(processTerminated, true);
});
//# sourceMappingURL=cancellation.test.js.map