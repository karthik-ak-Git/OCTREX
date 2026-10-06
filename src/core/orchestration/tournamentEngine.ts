import { VerificationReport } from '../types/verification.js';
import { VerificationEngine } from '../verification/verificationEngine.js';

export interface CandidateSolution {
  coderId: string;
  solutionName: string;
  patchSummary: string;
  workspaceOrWorktreePath: string;
}

export interface TournamentEvaluation {
  winnerCoderId?: string;
  winnerSolutionName?: string;
  reportCard: string;
  candidateReports: Map<string, VerificationReport>;
}

export class TournamentEngine {
  /**
   * Evaluates competing candidate solutions based on empirical verification evidence.
   */
  public static async evaluateCandidates(
    taskId: string,
    candidates: CandidateSolution[]
  ): Promise<TournamentEvaluation> {
    const candidateReports = new Map<string, VerificationReport>();
    let winnerId: string | undefined;
    let winnerName: string | undefined;
    let highestScore = -1;

    for (const cand of candidates) {
      const report = await VerificationEngine.evaluate({
        taskId: `${taskId}_${cand.coderId}`,
        workspacePath: cand.workspaceOrWorktreePath,
      });

      candidateReports.set(cand.coderId, report);

      // Score based on real evidence: build pass (30) + typecheck pass (20) + test ratio (50)
      let score = 0;
      if (report.checks.buildPassed) score += 30;
      if (report.checks.typecheckPassed) score += 20;
      if (report.checks.testsTotal > 0) {
        score += (report.checks.testsPassed / report.checks.testsTotal) * 50;
      }

      if (score > highestScore && report.status === 'VERIFIED') {
        highestScore = score;
        winnerId = cand.coderId;
        winnerName = cand.solutionName;
      }
    }

    const reportCard = [
      '==================================================',
      'OCTREX CODE V4 — TOURNAMENT MODE EVALUATION',
      '==================================================',
      `Candidates Evaluated: ${candidates.length}`,
      `Winning Candidate   : ${winnerName || 'None passed full verification'} (${winnerId || 'N/A'})`,
      '==================================================',
    ].join('\n');

    return {
      winnerCoderId: winnerId,
      winnerSolutionName: winnerName,
      reportCard,
      candidateReports,
    };
  }
}
