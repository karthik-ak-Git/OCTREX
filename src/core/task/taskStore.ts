import { TaskDefinition, TaskStatus, AgentRole } from '../types/agent.js';

export class TaskStore {
  private tasks: Map<string, TaskDefinition> = new Map();

  public createTask(userPrompt: string, workspacePath: string): TaskDefinition {
    const id = `task_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
    const now = new Date().toISOString();

    const task: TaskDefinition = {
      id,
      userPrompt,
      workspacePath,
      status: 'RECEIVED',
      createdAt: now,
      updatedAt: now,
    };

    this.tasks.set(id, task);
    return task;
  }

  public getTask(id: string): TaskDefinition | undefined {
    return this.tasks.get(id);
  }

  public updateTaskStatus(id: string, status: TaskStatus, activeAgent?: AgentRole): TaskDefinition | undefined {
    const task = this.tasks.get(id);
    if (!task) return undefined;

    task.status = status;
    if (activeAgent) task.activeAgent = activeAgent;
    task.updatedAt = new Date().toISOString();

    return task;
  }

  public listTasks(): TaskDefinition[] {
    return Array.from(this.tasks.values());
  }

  public clear(): void {
    this.tasks.clear();
  }
}
