/**
 * OCTREX CODE V4 — Frozen Backend Contract & Schema Specifications
 */

import { TaskDefinition, TaskStatus, AgentRole, UIEventType, UIEventPayload } from '../types/agent.js';
import { ModelDescriptor, ProviderHealthStatus, NormalizedError, ToolCall, ToolResult } from '../types/gateway.js';
import { VerificationReport } from '../types/verification.js';
import { RiskTier } from '../types/tools.js';

export interface FrozenSessionContract {
  sessionId: string;
  workspacePath: string;
  activeTaskId?: string;
  historyTasks: TaskDefinition[];
}

export interface FrozenProjectContract {
  workspacePath: string;
  framework?: string;
  buildCommands: string[];
  testCommands: string[];
  conventions: string[];
}

export interface FrozenProviderContract {
  providerId: string;
  displayName: string;
  health: ProviderHealthStatus;
  models: ModelDescriptor[];
}

export interface FrozenDiffContract {
  taskId: string;
  diff: string;
  filesChanged: string[];
}

export interface FrozenVerificationContract {
  taskId: string;
  report: VerificationReport;
}

export interface FrozenCheckpointContract {
  checkpointId: string;
  message: string;
  timestamp: string;
}

export interface FrozenRealtimeEventContract {
  eventType: UIEventType;
  payload: UIEventPayload;
}
