import { NormalizedChatRequest, NormalizedError } from '../types/gateway.js';

export class CloudConsentGuard {
  private static grantedWorkspaces: Set<string> = new Set();

  public static grantConsent(workspacePath: string): void {
    this.grantedWorkspaces.add(this.normalizePath(workspacePath));
  }

  public static revokeConsent(workspacePath: string): void {
    this.grantedWorkspaces.delete(this.normalizePath(workspacePath));
  }

  public static hasConsent(workspacePath?: string): boolean {
    if (!workspacePath) return false;
    return this.grantedWorkspaces.has(this.normalizePath(workspacePath));
  }

  public static clearAll(): void {
    this.grantedWorkspaces.clear();
  }

  private static normalizePath(p: string): string {
    return p.toLowerCase().replace(/\\/g, '/').replace(/\/+$/, '');
  }

  /**
   * Evaluates whether a request contains project or repository code.
   */
  public static containsRepositoryCode(request: NormalizedChatRequest): boolean {
    const allText = request.messages.map((m) => m.content).join('\n');
    return (
      allText.includes('[Relevant Repository Code Context]') ||
      allText.includes('--- File:') ||
      allText.includes('Git Diff:') ||
      allText.includes('```') ||
      request.messages.some((m) => Boolean(m.toolCalls?.length || m.toolResults?.length))
    );
  }

  /**
   * Backend-enforced verification: Throws PERMISSION_DENIED error if sending repository code to cloud without consent.
   */
  public static verifyConsent(
    request: NormalizedChatRequest,
    isLocal: boolean,
    workspacePath?: string
  ): { allowed: boolean; error?: NormalizedError } {
    // Local providers (e.g. Ollama, local NIM on 127.0.0.1) never require cloud consent
    if (isLocal) {
      return { allowed: true };
    }

    // Requests not containing repository code are allowed (e.g. general questions)
    if (!this.containsRepositoryCode(request)) {
      return { allowed: true };
    }

    // Verify backend consent has been granted for this workspace
    if (workspacePath && this.hasConsent(workspacePath)) {
      return { allowed: true };
    }

    const normError: NormalizedError = {
      code: 'PERMISSION_DENIED',
      message: 'Cloud-code consent required: sending repository code to cloud AI providers is blocked until explicit consent is granted in the backend.',
      providerId: request.providerId,
      modelId: request.modelId,
      retryable: false,
    };

    return {
      allowed: false,
      error: normError,
    };
  }
}
