import { ICancellationToken } from '../types/agent.js';
export declare class CancellationToken implements ICancellationToken {
    isCancelled: boolean;
    cancelReason?: string;
    private callbacks;
    cancel(reason?: string): void;
    onCancelled(callback: (reason: string) => void): void;
    throwIfCancelled(): void;
}
export declare class TaskCancellationManager {
    private activeTokens;
    getOrCreateToken(taskId: string): CancellationToken;
    handleProviderDisconnect(taskId: string, providerId: string): void;
    cancelTask(taskId: string, reason?: string): void;
    removeToken(taskId: string): void;
}
//# sourceMappingURL=cancellationToken.d.ts.map