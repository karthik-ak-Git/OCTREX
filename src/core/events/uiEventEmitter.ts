import { UIEventType, UIEventPayload } from '../types/agent.js';

export type UIEventListener = (event: UIEventPayload) => void;

export class UIEventEmitter {
  private listeners: Map<UIEventType | '*', Set<UIEventListener>> = new Map();

  public subscribe(eventType: UIEventType | '*', listener: UIEventListener): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }

    const set = this.listeners.get(eventType)!;
    set.add(listener);

    return () => {
      set.delete(listener);
    };
  }

  public emit(taskId: string, eventType: UIEventType, data: Record<string, any>): UIEventPayload {
    const payload: UIEventPayload = {
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
        } catch (err) {
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
        } catch (err) {
          console.error(`Wildcard UIEventListener error on event ${eventType}:`, err);
        }
      }
    }

    return payload;
  }
}
