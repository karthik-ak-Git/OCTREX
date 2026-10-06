import { ICancellationToken } from '../types/agent.js';

export class CancellationToken implements ICancellationToken {
  public isCancelled = false;
  public cancelReason?: string;

  private callbacks: Array<(reason: string) => void> = [];

  public cancel(reason = 'Task explicitly cancelled'): void {
    if (this.isCancelled) return;

    this.isCancelled = true;
    this.cancelReason = reason;

    for (const callback of this.callbacks) {
      try {
        callback(reason);
      } catch (err) {
        console.error('Error during cancellation callback execution:', err);
      }
    }
  }

  public onCancelled(callback: (reason: string) => void): void {
    if (this.isCancelled && this.cancelReason) {
      callback(this.cancelReason);
      return;
    }
    this.callbacks.push(callback);
  }

  public throwIfCancelled(): void {
    if (this.isCancelled) {
      throw new Error(`Operation cancelled: ${this.cancelReason || 'Unknown reason'}`);
    }
  }
}

export class TaskCancellationManager {
  private activeTokens: Map<string, CancellationToken> = new Map();

  public getOrCreateToken(taskId: string): CancellationToken {
    let token = this.activeTokens.get(taskId);
    if (!token) {
      token = new CancellationToken();
      this.activeTokens.set(taskId, token);
    }
    return token;
  }

  public handleProviderDisconnect(taskId: string, providerId: string): void {
    const token = this.activeTokens.get(taskId);
    if (token) {
      token.cancel(`Provider disconnect detected on provider '${providerId}'. Operation aborted to prevent zombie processes.`);
    }
  }

  public cancelTask(taskId: string, reason = 'User requested cancellation'): void {
    const token = this.activeTokens.get(taskId);
    if (token) {
      token.cancel(reason);
    }
  }

  public removeToken(taskId: string): void {
    this.activeTokens.delete(taskId);
  }
}
