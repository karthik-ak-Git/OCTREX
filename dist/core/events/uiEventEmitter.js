export class UIEventEmitter {
    listeners = new Map();
    subscribe(eventType, listener) {
        if (!this.listeners.has(eventType)) {
            this.listeners.set(eventType, new Set());
        }
        const set = this.listeners.get(eventType);
        set.add(listener);
        return () => {
            set.delete(listener);
        };
    }
    emit(taskId, eventType, data) {
        const payload = {
            taskId,
            eventType,
            timestamp: new Date().toISOString(),
            data,
        };
        // Notify specific event type listeners
        const specificListeners = this.listeners.get(eventType);
        if (specificListeners) {
            for (const listener of specificListeners) {
                try {
                    listener(payload);
                }
                catch (err) {
                    console.error(`UIEventListener error on event ${eventType}:`, err);
                }
            }
        }
        // Notify wildcard listeners
        const wildcardListeners = this.listeners.get('*');
        if (wildcardListeners) {
            for (const listener of wildcardListeners) {
                try {
                    listener(payload);
                }
                catch (err) {
                    console.error(`Wildcard UIEventListener error on event ${eventType}:`, err);
                }
            }
        }
        return payload;
    }
}
//# sourceMappingURL=uiEventEmitter.js.map