import { RiskTier, ExecutionResult } from '../types/tools.js';

export interface CheckpointHandler {
  createCheckpoint(message: string): Promise<ExecutionResult<{ checkpointId: string }>>;
}

export class CheckpointProtectionGuard {
  private checkpointHandler?: CheckpointHandler;

  constructor(handler?: CheckpointHandler) {
    this.checkpointHandler = handler;
  }

  public setHandler(handler: CheckpointHandler) {
    this.checkpointHandler = handler;
  }

  /**
   * Evaluates action risk tier and enforces Git checkpoint creation before execution.
   */
  public async ensureCheckpointBeforeExecution(
    actionName: string,
    riskTier: RiskTier,
    taskId: string
  ): Promise<{ proceed: boolean; checkpointId?: string; reason?: string }> {
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
    } catch (err: any) {
      return {
        proceed: false,
        reason: `Checkpoint creation exception: ${err.message || String(err)}`,
      };
    }
  }
}
