import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CheckpointProtectionGuard } from '../core/security/checkpointProtection.js';
test('CheckpointProtectionGuard allows LOW risk actions without checkpointing', async () => {
    const guard = new CheckpointProtectionGuard();
    const decision = await guard.ensureCheckpointBeforeExecution('file_read', 'LOW', 'task-101');
    assert.equal(decision.proceed, true);
});
test('CheckpointProtectionGuard triggers automatic Git checkpoint for MEDIUM and HIGH risk operations', async () => {
    let checkpointMessageCreated = '';
    const mockHandler = {
        async createCheckpoint(msg) {
            checkpointMessageCreated = msg;
            return {
                success: true,
                data: { checkpointId: 'chk-git-99' },
                executionTimeMs: 12,
            };
        },
    };
    const guard = new CheckpointProtectionGuard(mockHandler);
    const decision = await guard.ensureCheckpointBeforeExecution('file_patch', 'MEDIUM', 'task-102');
    assert.equal(decision.proceed, true);
    assert.equal(decision.checkpointId, 'chk-git-99');
    assert.ok(checkpointMessageCreated.includes('file_patch'));
    const highDecision = await guard.ensureCheckpointBeforeExecution('file_delete', 'HIGH', 'task-103');
    assert.equal(highDecision.proceed, true);
    assert.equal(highDecision.checkpointId, 'chk-git-99');
});
test('CheckpointProtectionGuard aborts execution if safety checkpoint fails', async () => {
    const failingHandler = {
        async createCheckpoint() {
            return {
                success: false,
                error: 'Git dirty state cannot lock repository',
                executionTimeMs: 5,
            };
        },
    };
    const guard = new CheckpointProtectionGuard(failingHandler);
    const decision = await guard.ensureCheckpointBeforeExecution('file_patch', 'HIGH', 'task-104');
    assert.equal(decision.proceed, false);
    assert.ok(decision.reason?.includes('Failed to create safety checkpoint'));
});
//# sourceMappingURL=checkpointProtection.test.js.map