'use client';

import React from 'react';
import { History } from 'lucide-react';
import type { VerificationResult } from '@/lib/backend/types';
import { VerificationStatus } from './VerificationStatus';

interface RepairAttemptTimelineProps {
  runs: VerificationResult[];
  selectedId?: string | null;
  onSelect?: (verificationId: string) => void;
  maxRepairAttempts?: number;
}

export const RepairAttemptTimeline: React.FC<RepairAttemptTimelineProps> = ({
  runs,
  selectedId,
  onSelect,
  maxRepairAttempts,
}) => {
  if (!runs || runs.length === 0) {
    return (
      <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 text-xs text-zinc-500 text-center">
        No verification runs yet. Repair cycles are bounded
        {typeof maxRepairAttempts === 'number' ? ` (max ${maxRepairAttempts} attempts)` : ''} — never infinite.
      </div>
    );
  }
  const ordered = [...runs].sort((a, b) => a.created_at - b.created_at);
  return (
    <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 space-y-3">
      <div className="flex items-center justify-between gap-2">
        <span className="flex items-center gap-1.5 text-xs font-semibold text-zinc-300">
          <History className="w-3.5 h-3.5 text-zinc-500" />
          Verification & repair timeline ({ordered.length} run{ordered.length === 1 ? '' : 's'})
        </span>
        {typeof maxRepairAttempts === 'number' && (
          <span className="text-[10px] font-mono text-zinc-500">
            repair budget: {maxRepairAttempts}
          </span>
        )}
      </div>
      <ol className="space-y-2">
        {ordered.map((run, idx) => {
          const isSelected = run.verification_id === selectedId;
          return (
            <li key={run.verification_id}>
              <button
                onClick={() => onSelect?.(run.verification_id)}
                className={`w-full text-left p-2.5 rounded-lg border transition-all ${
                  isSelected
                    ? 'border-blue-500/40 bg-blue-500/5'
                    : 'border-zinc-800 bg-zinc-900/40 hover:border-zinc-700'
                }`}
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="text-[11px] font-mono text-zinc-400">
                    #{idx + 1} · {run.verification_id.slice(0, 12)}… · attempt {run.repair_attempt}
                  </span>
                  <VerificationStatus status={run.status} compact />
                </div>
                <div className="text-[10px] text-zinc-600 font-mono mt-1">
                  {new Date(run.created_at).toLocaleString()} · {run.checks.length} checks ·{' '}
                  {(run.confidence * 100).toFixed(0)}% confidence
                </div>
              </button>
            </li>
          );
        })}
      </ol>
    </div>
  );
};
