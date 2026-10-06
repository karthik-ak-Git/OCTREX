export class CheckpointProtectionGuard {
    checkpointHandler;
    constructor(handler) {
        this.checkpointHandler = handler;
    }
    setHandler(handler) {
        this.checkpointHandler = handler;
    }
    /**
     * Evaluates action risk tier and enforces Git checkpoint creation before execution.
     */
    async ensureCheckpointBeforeExecution(actionName, riskTier, taskId) {
        // LOW risk actions (reads, searches, logs) proceed without checkpoint
        if (riskTier === 'LOW') {
            return { proceed: true };
        }
        if (!this.checkpointHandler) {
            // If no git handler registered, proceed with warning or default protection
            return {
                proceed: true,
                reason: 'No checkpoint handler registered; executed under fallback policy.',
            };
        }
        try {
            const msg = `Auto-checkpoint before agent execution: ${actionName} (Task: ${taskId})`;
            const result = await this.checkpointHandler.createCheckpoint(msg);
            if (!result.success) {
                return {
                    proceed: false,
                    reason: `Failed to create safety checkpoint: ${result.error}`,
                };
            }
            return {
                proceed: true,
                checkpointId: result.data?.checkpointId,
            };
        }
        catch (err) {
            return {
                proceed: false,
                reason: `Checkpoint creation exception: ${err.message || String(err)}`,
            };
        }
    }
}
//# sourceMappingURL=checkpointProtection.js.map