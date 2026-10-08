import { EventEnvelope } from './types';

export type EventCallback = (event: EventEnvelope) => void;

export class OctrexEventSubscriber {
  private eventSource: EventSource | null = null;
  private listeners: Set<EventCallback> = new Set();

  constructor(private url = '/api/events') {}

  connect(): void {
    if (typeof window === 'undefined') return;
    if (this.eventSource) return;

    this.eventSource = new EventSource(this.url);

    this.eventSource.onmessage = (e) => {
      try {
        const envelope: EventEnvelope = JSON.parse(e.data);
        this.listeners.forEach((callback) => callback(envelope));
      } catch (err) {
        console.error('Failed to parse backend SSE event envelope:', err);
      }
    };

    this.eventSource.onerror = (err) => {
      console.warn('Backend SSE event stream connection error:', err);
    };
  }

  subscribe(callback: EventCallback): () => void {
    this.listeners.add(callback);
    if (!this.eventSource) {
      this.connect();
    }
    return () => {
      this.listeners.delete(callback);
    };
  }

  disconnect(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
    this.listeners.clear();
  }
}

export const eventSubscriber = new OctrexEventSubscriber();
