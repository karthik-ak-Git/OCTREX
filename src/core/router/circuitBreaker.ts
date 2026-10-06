export type CircuitState = 'CLOSED' | 'OPEN' | 'HALF_OPEN';

export interface CircuitBreakerOptions {
  failureThreshold?: number;
  cooldownPeriodMs?: number;
}

export class CircuitBreaker {
  public providerId: string;
  public state: CircuitState = 'CLOSED';

  private failureCount = 0;
  private failureThreshold: number;
  private cooldownPeriodMs: number;
  private lastFailureTime = 0;

  constructor(providerId: string, options?: CircuitBreakerOptions) {
    this.providerId = providerId;
    this.failureThreshold = options?.failureThreshold || 3;
    this.cooldownPeriodMs = options?.cooldownPeriodMs || 30000;
  }

  public isAvailable(): boolean {
    if (this.state === 'CLOSED') {
      return true;
    }

    if (this.state === 'OPEN') {
      const now = Date.now();
      if (now - this.lastFailureTime > this.cooldownPeriodMs) {
        this.state = 'HALF_OPEN';
        return true;
      }
      return false;
    }

    // HALF_OPEN allows single probe
    return true;
  }

  public recordSuccess(): void {
    this.failureCount = 0;
    this.state = 'CLOSED';
  }

  public recordFailure(): void {
    this.failureCount += 1;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.failureThreshold || this.state === 'HALF_OPEN') {
      this.state = 'OPEN';
    }
  }

  public reset(): void {
    this.failureCount = 0;
    this.state = 'CLOSED';
    this.lastFailureTime = 0;
  }
}
