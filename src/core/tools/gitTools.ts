import { TerminalTools } from './terminalTools.js';
import { ExecutionResult } from '../types/tools.js';

export class GitTools {
  private workspacePath: string;

  constructor(workspacePath: string) {
    this.workspacePath = workspacePath;
  }

  public async getStatus(): Promise<ExecutionResult<{ isClean: boolean; branch: string; rawStatus: string }>> {
    const res = await TerminalTools.execute({
      taskId: 'git-status',
      riskTier: 'LOW',
      command: 'git status --porcelain -b',
      cwd: this.workspacePath,
    });

    if (!res.success || !res.data) {
      return { success: false, error: res.error, executionTimeMs: res.executionTimeMs };
    }

    const lines = res.data.stdout.trim().split('\n');
    const branchLine = lines[0] || '';
    const branch = branchLine.replace('## ', '').split('...')[0].trim();
    const isClean = lines.length <= 1 || (lines.length === 2 && !lines[1].trim());

    return {
      success: true,
      data: { isClean, branch, rawStatus: res.data.stdout },
      executionTimeMs: res.executionTimeMs,
    };
  }

  public async getDiff(): Promise<ExecutionResult<{ diff: string }>> {
    const res = await TerminalTools.execute({
      taskId: 'git-diff',
      riskTier: 'LOW',
      command: 'git diff',
      cwd: this.workspacePath,
    });

    return {
      success: res.success,
      data: { diff: res.data?.stdout || '' },
      error: res.error,
      executionTimeMs: res.executionTimeMs,
    };
  }

  public async createCheckpoint(message: string): Promise<ExecutionResult<{ checkpointId: string }>> {
    const checkpointId = `chk_${Date.now()}`;
    const commitMsg = `checkpoint: [${checkpointId}] ${message}`;

    // Add and commit current state as safe checkpoint
    const addRes = await TerminalTools.execute({
      taskId: 'git-checkpoint-add',
      riskTier: 'MEDIUM',
      command: 'git add .',
      cwd: this.workspacePath,
    });

    if (!addRes.success) {
      return { success: false, error: addRes.error, executionTimeMs: addRes.executionTimeMs };
    }

    const commitRes = await TerminalTools.execute({
      taskId: 'git-checkpoint-commit',
      riskTier: 'MEDIUM',
      command: `git commit -m "${commitMsg}" --allow-empty`,
      cwd: this.workspacePath,
    });

    return {
      success: commitRes.success,
      data: { checkpointId },
      error: commitRes.error,
      executionTimeMs: commitRes.executionTimeMs,
    };
  }

  public async rollbackToCheckpoint(checkpointHashOrHead = 'HEAD~1'): Promise<ExecutionResult> {
    const res = await TerminalTools.execute({
      taskId: 'git-rollback',
      riskTier: 'HIGH',
      command: `git reset --soft ${checkpointHashOrHead}`,
      cwd: this.workspacePath,
    });

    return {
      success: res.success,
      error: res.error,
      executionTimeMs: res.executionTimeMs,
    };
  }
}
