import { TerminalTools } from './terminalTools.js';
import { ExecutionResult } from '../types/tools.js';

export interface DevCheckResult {
  passed: boolean;
  output: string;
  durationMs: number;
}

export class DevTools {
  private workspacePath: string;

  constructor(workspacePath: string) {
    this.workspacePath = workspacePath;
  }

  public async runTypecheck(): Promise<DevCheckResult> {
    const res = await TerminalTools.execute({
      taskId: 'dev-typecheck',
      riskTier: 'LOW',
      command: 'npm run typecheck',
      cwd: this.workspacePath,
    });

    return {
      passed: res.success,
      output: (res.data?.stdout || '') + (res.data?.stderr || ''),
      durationMs: res.executionTimeMs,
    };
  }

  public async runBuild(): Promise<DevCheckResult> {
    const res = await TerminalTools.execute({
      taskId: 'dev-build',
      riskTier: 'LOW',
      command: 'npm run build',
      cwd: this.workspacePath,
    });

    return {
      passed: res.success,
      output: (res.data?.stdout || '') + (res.data?.stderr || ''),
      durationMs: res.executionTimeMs,
    };
  }

  public async runTests(targetTestFile?: string): Promise<DevCheckResult> {
    const cmd = targetTestFile ? `npx tsx --test ${targetTestFile}` : 'npm test';

    const res = await TerminalTools.execute({
      taskId: 'dev-test',
      riskTier: 'LOW',
      command: cmd,
      cwd: this.workspacePath,
    });

    return {
      passed: res.success,
      output: (res.data?.stdout || '') + (res.data?.stderr || ''),
      durationMs: res.executionTimeMs,
    };
  }
}
