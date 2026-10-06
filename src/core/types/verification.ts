/**
 * OCTREX CODE V4 - Verification Engine Contracts
 */

export interface VerificationCheckSummary {
  buildPassed: boolean;
  testsPassed: number;
  testsTotal: number;
  typecheckPassed: boolean;
  lintPassed: boolean;
  reviewerApproved: boolean;
  acceptanceCriteriaMet: boolean;
}

export interface VerificationReport {
  taskId: string;
  timestamp: string;
  status: 'VERIFIED' | 'FAILED';
  checks: VerificationCheckSummary;
  evidenceLogs: {
    buildOutput?: string;
    testSummary?: string;
    reviewerComments?: string;
    typecheckOutput?: string;
  };
}
