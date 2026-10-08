'use client';

import React from 'react';
import { Check, AlertTriangle, X, MinusCircle, HelpCircle, SkipForward } from 'lucide-react';
import type { VerificationCheck } from '@/lib/backend/types';

interface VerificationChecklistProps {
  checks: VerificationCheck[];
}

const CHECK_META: Record<string, { className: string; Icon: any; symbol: string }> = {
  PASSED: { className: 'text-emerald-400', Icon: Check, symbol: '✓' },
  WARNING: { className: 'text-amber-400', Icon: AlertTriangle, symbol: '!' },
  FAILED: { className: 'text-rose-400', Icon: X, symbol: '✗' },
  BLOCKED: { className: 'text-red-400', Icon: MinusCircle, symbol: '⊘' },
  UNKNOWN: { className: 'text-zinc-400', Icon: HelpCircle, symbol: '?' },
  SKIPPED: { className: 'text-zinc-500', Icon: SkipForward, symbol: '→' },
};

export const VerificationChecklist: React.FC<VerificationChecklistProps> = ({ checks }) => {
  if (!checks || checks.length === 0) {
    return (
      <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 text-xs text-zinc-500 text-center">
        No verification checks recorded. A task with no checks is NOT verified.
      </div>
    );
  }
  return (
    <ul className="space-y-2">
      {checks.map((check) => {
        const meta = CHECK_META[String(check.status)] ?? CHECK_META.UNKNOWN;
        const { Icon } = meta;
        return (
          <li
            key={check.id}
            className="p-3 rounded-lg border border-zinc-800 bg-zinc-950/60 space-y-1"
          >
            <div className="flex items-center justify-between gap-2">
              <span className={`flex items-center gap-1.5 text-xs font-semibold ${meta.className}`}>
                <Icon className="w-3.5 h-3.5" />
                <span className="font-mono">{meta.symbol}</span>
                <span className="text-zinc-200">{check.name}</span>
              </span>
              <span className="text-[10px] font-mono text-zinc-500 uppercase">
                {check.check_type} · {check.severity}
              </span>
            </div>
            <div className="text-[11px] font-mono text-zinc-400 pl-5">
              expected: <span className="text-zinc-300">{check.expected}</span>
              {' → '}actual: <span className="text-zinc-300">{check.actual}</span>
            </div>
            {check.message && (
              <div className="text-[11px] text-zinc-500 pl-5">{check.message}</div>
            )}
            {check.evidence_ref && (
              <div className="text-[10px] font-mono text-zinc-600 pl-5">
                evidence: {check.evidence_ref}
              </div>
            )}
          </li>
        );
      })}
    </ul>
  );
};
