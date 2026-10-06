import { test } from 'node:test';
import assert from 'node:assert/strict';
import { VerificationEngine } from '../core/verification/verificationEngine.js';
import { AutoRepairLoop } from '../core/verification/repairLoop.js';
import { ModelRouter } from '../core/router/modelRouter.js';
import { UniversalModelGateway } from '../core/gateway/universalGateway.js';
import { MockProviderAdapter } from '../core/gateway/mockAdapter.js';

test('VerificationEngine generates structured empirical verification report card', async () => {
  const report = await VerificationEngine.evaluate({
    taskId: 'task-verify-101',
    workspacePath: process.cwd(),
    reviewerApproved: true,
    acceptanceCriteriaCount: 3,
    acceptanceCriteriaMetCount: 3,
  });

  assert.equal(report.taskId, 'task-verify-101');
  assert.equal(report.status, 'VERIFIED');
  assert.equal(report.checks.buildPassed, true);
  assert.equal(report.checks.typecheckPassed, true);
  assert.equal(report.checks.reviewerApproved, true);
  assert.equal(report.checks.acceptanceCriteriaMet, true);

  const formatted = VerificationEngine.formatReportCard(report);
  assert.ok(formatted.includes('OCTREX CODE V4 — VERIFICATION REPORT CARD'));
  assert.ok(formatted.includes('BUILD        : PASS'));
  assert.ok(formatted.includes('STATUS       : VERIFIED'));
});

test('AutoRepairLoop completes and returns verified report', async () => {
  const gateway = new UniversalModelGateway();
  gateway.registerAdapter(new MockProviderAdapter());
  const router = new ModelRouter(gateway);

  const repairLoop = new AutoRepairLoop(router);
  const attempts: number[] = [];

  const finalReport = await repairLoop.runVerificationAndRepair('task-repair-102', process.cwd(), {
    maxAttempts: 2,
    onAttempt: (att) => {
      attempts.push(att);
    },
  });

  assert.equal(finalReport.status, 'VERIFIED');
  assert.ok(attempts.length >= 1);
});
