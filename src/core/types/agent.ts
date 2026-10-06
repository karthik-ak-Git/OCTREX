/**
 * OCTREX CODE V4 - Agent Orchestrator & Task Contracts
 */

export type AgentRole = 
  | 'MANAGER'
  | 'PLANNER'
  | 'CODER'
  | 'DEBUGGER'
  | 'TESTER'
  | 'REVIEWER';

export type TaskStatus = 
  | 'RECEIVED'
  | 'UNDERSTANDING'
  | 'REPOSITORY_ANALYSIS'
  | 'PLANNING'
  | 'IMPLEMENTING'
  | 'TESTING'
  | 'DEBUGGING'
  | 'REVIEWING'
  | 'VERIFIED'
  | 'FAILED'
  | 'CANCELLED';

export interface TaskDefinition {
  id: string;
  userPrompt: string;
  workspacePath: string;
  status: TaskStatus;
  createdAt: string;
  updatedAt: string;
  activeAgent?: AgentRole;
  currentModelId?: string;
  currentProviderId?: string;
}

export type UIEventType = 
  | 'task_updated'
  | 'agent_started'
  | 'agent_completed'
  | 'agent_failed'
  | 'tool_executed'
  | 'terminal_chunk'
  | 'diff_generated'
  | 'provider_health_changed'
  | 'model_fallback_occurred'
  | 'verification_completed'
  | 'checkpoint_created';

export interface UIEventPayload {
  taskId: string;
  eventType: UIEventType;
  timestamp: string;
  data: Record<string, any>;
}

export interface ICancellationToken {
  isCancelled: boolean;
  cancelReason?: string;
  onCancelled(callback: (reason: string) => void): void;
  throwIfCancelled(): void;
}
