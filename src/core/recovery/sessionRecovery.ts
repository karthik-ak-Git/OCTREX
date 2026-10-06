import { TaskStore } from '../task/taskStore.js';
import { TaskDefinition } from '../types/agent.js';

export interface RecoveryResult {
  recoveredTasksCount: number;
  activeTasksResetToFailed: number;
}

export class SessionRecoveryManager {
  private taskStore: TaskStore;

  constructor(taskStore: TaskStore) {
    this.taskStore = taskStore;
  }

  /**
   * Safely recovers session after unexpected shutdown without auto-executing dangerous commands.
   */
  public recoverSession(): RecoveryResult {
    const tasks = this.taskStore.listTasks();
    let activeResetCount = 0;

    for (const task of tasks) {
      if (
        task.status === 'IMPLEMENTING' ||
        task.status === 'TESTING' ||
        task.status === 'DEBUGGING' ||
        task.status === 'PLANNING'
      ) {
        // Reset in-flight tasks to COMPLETED_UNVERIFIED or FAILED so they don't blindly re-run unverified commands
        this.taskStore.updateTaskStatus(task.id, 'FAILED');
        activeResetCount += 1;
      }
    }

    return {
      recoveredTasksCount: tasks.length,
      activeTasksResetToFailed: activeResetCount,
    };
  }
}
