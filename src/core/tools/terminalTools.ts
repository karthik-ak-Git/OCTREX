import { spawn } from 'node:child_process';
import { ExecutionResult, TerminalExecuteOptions } from '../types/tools.js';
import { ICancellationToken } from '../types/agent.js';
import { PermissionModel } from '../security/permissionModel.js';

export interface TerminalOutput {
  stdout: string;
  stderr: string;
  exitCode: number | null;
}

export class TerminalTools {
  public static async execute(
    options: TerminalExecuteOptions,
    cancellationToken?: ICancellationToken
  ): Promise<ExecutionResult<TerminalOutput>> {
    const startTime = Date.now();

    // Risk Check
    const riskAnalysis = PermissionModel.analyzeCommand(options.command);
    const permission = PermissionModel.canExecute(riskAnalysis);
    if (!permission.allowed) {
      return {
        success: false,
        error: permission.reason,
        executionTimeMs: Date.now() - startTime,
      };
    }

    return new Promise<ExecutionResult<TerminalOutput>>((resolve) => {
      let stdoutBuffer = '';
      let stderrBuffer = '';
      let isSettled = false;

      const isWin = process.platform === 'win32';
      const shell = isWin ? 'powershell.exe' : '/bin/bash';
      const args = isWin ? ['-NoProfile', '-NonInteractive', '-Command', options.command] : ['-c', options.command];

      const child = spawn(shell, args, {
        cwd: options.cwd,
        env: { ...process.env },
      });

      // Cancellation listener
      if (cancellationToken) {
        cancellationToken.onCancelled(() => {
          if (!isSettled) {
            isSettled = true;
            child.kill('SIGKILL');
            resolve({
              success: false,
              error: `Terminal command cancelled: ${cancellationToken.cancelReason}`,
              executionTimeMs: Date.now() - startTime,
            });
          }
        });
      }

      // Timeout listener
      const timeoutMs = options.timeoutMs || 60000;
      const timeoutId = setTimeout(() => {
        if (!isSettled) {
          isSettled = true;
          child.kill('SIGKILL');
          resolve({
            success: false,
            error: `Terminal command timed out after ${timeoutMs}ms`,
            executionTimeMs: Date.now() - startTime,
          });
        }
      }, timeoutMs);

      child.stdout.on('data', (chunk: Buffer) => {
        const str = chunk.toString();
        stdoutBuffer += str;
        options.onStdoutChunk?.(str);
      });

      child.stderr.on('data', (chunk: Buffer) => {
        const str = chunk.toString();
        stderrBuffer += str;
        options.onStderrChunk?.(str);
      });

      child.on('close', (code: number | null) => {
        clearTimeout(timeoutId);
        if (!isSettled) {
          isSettled = true;
          resolve({
            success: code === 0,
            data: {
              stdout: stdoutBuffer,
              stderr: stderrBuffer,
              exitCode: code,
            },
            error: code !== 0 ? `Process exited with code ${code}` : undefined,
            executionTimeMs: Date.now() - startTime,
          });
        }
      });

      child.on('error', (err: Error) => {
        clearTimeout(timeoutId);
        if (!isSettled) {
          isSettled = true;
          resolve({
            success: false,
            error: err.message,
            executionTimeMs: Date.now() - startTime,
          });
        }
      });
    });
  }
}
