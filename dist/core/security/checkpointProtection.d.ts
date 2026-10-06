import { RiskTier, ExecutionResult } from '../types/tools.js';
export interface CheckpointHandler {
    createCheckpoint(message: string): Promise<ExecutionResult<{
        checkpointId: string;
    }>>;
}
export declare class CheckpointProtectionGuard {
    private checkpointHandler?;
    constructor(handler?: CheckpointHandler);
    setHandler(handler: CheckpointHandler): void;
    /**
     * Evaluates action risk tier and enforces Git checkpoint creation before execution.
     */
    ensureCheckpointBeforeExecution(actionName: string, riskTier: RiskTier, taskId: string): Promise<{
        proceed: boolean;
        checkpointId?: string;
        reason?: string;
    }>;
}
//# sourceMappingURL=checkpointProtection.d.ts.map