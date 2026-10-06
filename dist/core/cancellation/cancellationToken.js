export class CancellationToken {
    isCancelled = false;
    cancelReason;
    callbacks = [];
    cancel(reason = 'Task explicitly cancelled') {
        if (this.isCancelled)
            return;
        this.isCancelled = true;
        this.cancelReason = reason;
        for (const callback of this.callbacks) {
            try {
                callback(reason);
            }
            catch (err) {
                console.error('Error during cancellation callback execution:', err);
            }
        }
    }
    onCancelled(callback) {
        if (this.isCancelled && this.cancelReason) {
            callback(this.cancelReason);
            return;
        }
        this.callbacks.push(callback);
    }
    throwIfCancelled() {
        if (this.isCancelled) {
            throw new Error(`Operation cancelled: ${this.cancelReason || 'Unknown reason'}`);
        }
    }
}
export class TaskCancellationManager {
    activeTokens = new Map();
    getOrCreateToken(taskId) {
        let token = this.activeTokens.get(taskId);
        if (!token) {
            token = new CancellationToken();
            this.activeTokens.set(taskId, token);
        }
        return token;
    }
    handleProviderDisconnect(taskId, providerId) {
        const token = this.activeTokens.get(taskId);
        if (token) {
            token.cancel(`Provider disconnect detected on provider '${providerId}'. Operation aborted to prevent zombie processes.`);
        }
    }
    cancelTask(taskId, reason = 'User requested cancellation') {
        const token = this.activeTokens.get(taskId);
        if (token) {
            token.cancel(reason);
        }
    }
    removeToken(taskId) {
        this.activeTokens.delete(taskId);
    }
}
//# sourceMappingURL=cancellationToken.js.map