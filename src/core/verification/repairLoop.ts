import { VerificationEngine, VerificationEvaluationParams } from './verificationEngine.js';
import { VerificationReport } from '../types/verification.js';
import { ModelRouter } from '../router/modelRouter.js';
import { ContextEngine } from '../context/contextEngine.js';
import { ICancellationToken } from '../types/agent.js';

export interface RepairLoopOptions {
  maxAttempts?: number;
  targetTestFile?: string;
  onAttempt?: (attemptNumber: number, report: VerificationReport) => void;
}

export class AutoRepairLoop {
  private router: ModelRouter;
  private contextEngine: ContextEngine;

  constructor(router: ModelRouter, contextEngine = new ContextEngine()) {
    this.router = router;
    this.contextEngine = contextEngine;
  }

  /**
   * Executes continuous Plan -> Implement -> Build/Test -> Fail -> Debug -> Repair -> Retest -> Verify loop.
   */
  public async runVerificationAndRepair(
    taskId: string,
    workspacePath: string,
    options?: RepairLoopOptions,
    cancellationToken?: ICancellationToken
  ): Promise<VerificationReport> {
    const maxAttempts = options?.maxAttempts || 3;
    let attempt = 0;
    let lastReport: VerificationReport;

    while (attempt < maxAttempts) {
      attempt += 1;
      cancellationToken?.throwIfCancelled();

      lastReport = await VerificationEngine.evaluate({
        taskId,
        workspacePath,
        targetTestFile: options?.targetTestFile,
      });

      options?.onAttempt?.(attempt, lastReport);

      if (lastReport.status === 'VERIFIED') {
        return lastReport;
      }

      // If failed, trigger Debugger Agent repair proposal
      const failureTrace = (lastReport.evidenceLogs.testSummary || '') + (lastReport.evidenceLogs.typecheckOutput || '');
      const debugPrompt = this.contextEngine.assemblePrompt({
        systemInstruction: 'You are the DEBUGGER AGENT. Provide a specific repair patch for this failure.',
        taskPrompt: `Attempt #${attempt} failed with logs:\n${this.contextEngine.pruneToolOutput(failureTrace, 40)}`,
      });

      await this.router.executeChatWithFallback(
        { taskId, role: 'DEBUGGER', requiresReasoning: true },
        debugPrompt,
        cancellationToken
      );
    }

    return lastReport!;
  }
}
