import { UIEventType, UIEventPayload } from '../types/agent.js';
export type UIEventListener = (event: UIEventPayload) => void;
export declare class UIEventEmitter {
    private listeners;
    subscribe(eventType: UIEventType | '*', listener: UIEventListener): () => void;
    emit(taskId: string, eventType: UIEventType, data: Record<string, any>): UIEventPayload;
}
//# sourceMappingURL=uiEventEmitter.d.ts.map