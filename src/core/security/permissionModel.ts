import { RiskTier } from '../types/tools.js';

export interface CommandRiskAnalysis {
  command: string;
  riskTier: RiskTier;
  reason: string;
  isBlocked: boolean;
}

export class PermissionModel {
  // Dangerous commands that must be strictly blocked or gated as HIGH risk
  private static DANGEROUS_PATTERNS = [
    { pattern: /(?:^|\s)(?:rm\s+-rf\s+[/~]|rmdir\s+\/s\s+\/q\s+[c-zC-Z]:\\)/i, reason: 'Destructive root or root-drive deletion', blocked: true },
    { pattern: /(?:^|\s)(?:format\s+[a-zA-Z]:|mkfs|diskpart)/i, reason: 'Disk format command', blocked: true },
    { pattern: /(?:^|\s)(?:shutdown|reboot|init\s+0)/i, reason: 'System shutdown/restart', blocked: true },
    { pattern: /(?:^|\s)(?:git\s+push\s+(?:.*--force|-f))/i, reason: 'Destructive force push to remote Git repository', blocked: false, risk: 'HIGH' as const },
    { pattern: /(?:^|\s)(?:git\s+reset\s+--hard)/i, reason: 'Hard Git reset discarding uncommitted work', blocked: false, risk: 'HIGH' as const },
    { pattern: /(?:^|\s)(?:curl|wget)\s+.*\|\s*(?:bash|sh|cmd|powershell)/i, reason: 'Piping untrusted remote scripts to shell', blocked: true },
    { pattern: /(?:^|\s)(?:chmod\s+777|icacls\s+.*\/grant\s+Everyone:\(F\))/i, reason: 'Unsafe global permission elevation', blocked: false, risk: 'HIGH' as const },
    { pattern: /(?:^|\s)(?:reg\s+delete|regedit)/i, reason: 'System registry tampering', blocked: true },
  ];

  private static MEDIUM_PATTERNS = [
    { pattern: /(?:^|\s)(?:npm\s+install|npm\s+i|yarn\s+add|pip\s+install|cargo\s+add)/i, reason: 'Package dependency installation' },
    { pattern: /(?:^|\s)(?:git\s+commit|git\s+checkout|git\s+branch)/i, reason: 'Workspace Git state change' },
    { pattern: /(?:^|\s)(?:mkdir|touch|mv|cp|copy|move)/i, reason: 'File creation or movement' },
  ];

  private static LOW_PATTERNS = [
    { pattern: /(?:^|\s)(?:npm\s+test|npm\s+run\s+test|npx\s+tsx|pytest|cargo\s+test|go\s+test)/i, reason: 'Project test execution' },
    { pattern: /(?:^|\s)(?:npm\s+run\s+build|tsc|tsc\s+--noEmit|eslint|prettier)/i, reason: 'Project build or lint verification' },
    { pattern: /(?:^|\s)(?:git\s+status|git\s+diff|git\s+log|dir|ls|cat|type)/i, reason: 'Safe workspace inspection' },
  ];

  /**
   * Analyzes a shell command and determines its RiskTier and safety classification.
   */
  public static analyzeCommand(command: string): CommandRiskAnalysis {
    const trimmed = command.trim();

    // Check dangerous patterns
    for (const d of PermissionModel.DANGEROUS_PATTERNS) {
      if (d.pattern.test(trimmed)) {
        return {
          command: trimmed,
          riskTier: 'HIGH',
          reason: d.reason,
          isBlocked: Boolean(d.blocked),
        };
      }
    }

    // Check medium patterns
    for (const m of PermissionModel.MEDIUM_PATTERNS) {
      if (m.pattern.test(trimmed)) {
        return {
          command: trimmed,
          riskTier: 'MEDIUM',
          reason: m.reason,
          isBlocked: false,
        };
      }
    }

    // Check low patterns
    for (const l of PermissionModel.LOW_PATTERNS) {
      if (l.pattern.test(trimmed)) {
        return {
          command: trimmed,
          riskTier: 'LOW',
          reason: l.reason,
          isBlocked: false,
        };
      }
    }

    // Default unknown command is classified as MEDIUM risk
    return {
      command: trimmed,
      riskTier: 'MEDIUM',
      reason: 'Standard workspace command',
      isBlocked: false,
    };
  }

  /**
   * Checks if an action is allowed based on user policy or auto-approval.
   */
  public static canExecute(
    analysis: CommandRiskAnalysis,
    autoApproveMedium = true,
    userApprovedHigh = false
  ): { allowed: boolean; reason: string } {
    if (analysis.isBlocked) {
      return {
        allowed: false,
        reason: `Command blocked by security policy: ${analysis.reason}`,
      };
    }

    if (analysis.riskTier === 'LOW') {
      return { allowed: true, reason: 'LOW risk action auto-permitted' };
    }

    if (analysis.riskTier === 'MEDIUM') {
      if (autoApproveMedium) {
        return { allowed: true, reason: 'MEDIUM risk action auto-approved by workspace policy' };
      }
      return { allowed: false, reason: 'MEDIUM risk action requires confirmation' };
    }

    if (analysis.riskTier === 'HIGH') {
      if (userApprovedHigh) {
        return { allowed: true, reason: 'HIGH risk action explicitly confirmed by user' };
      }
      return { allowed: false, reason: 'HIGH risk action requires explicit user permission' };
    }

    return { allowed: false, reason: 'Unknown risk classification' };
  }
}
