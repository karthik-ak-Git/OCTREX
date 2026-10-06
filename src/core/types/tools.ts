/**
 * OCTREX CODE V4 - Tool System Contracts
 */

export type RiskTier = 'LOW' | 'MEDIUM' | 'HIGH';

export interface BaseToolExecutionOptions {
  taskId: string;
  riskTier: RiskTier;
  allowAutoExecute?: boolean;
}

export interface FileReadOptions extends BaseToolExecutionOptions {
  filePath: string;
  startLine?: number;
  endLine?: number;
}

export interface FilePatchOptions extends BaseToolExecutionOptions {
  filePath: string;
  startLine: number;
  endLine: number;
  targetContent: string;
  replacementContent: string;
  description: string;
}

export interface TerminalExecuteOptions extends BaseToolExecutionOptions {
  command: string;
  cwd: string;
  timeoutMs?: number;
  onStdoutChunk?: (chunk: string) => void;
  onStderrChunk?: (chunk: string) => void;
}

export interface GitCheckpointOptions extends BaseToolExecutionOptions {
  message: string;
}

export interface ExecutionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  executionTimeMs: number;
}
