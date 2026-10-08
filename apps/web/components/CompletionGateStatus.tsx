'use client';

import React from 'react';
import { LockOpen, Wrench, UserCheck, OctagonX } from 'lucide-react';
import type { CompletionDecision } from '@/lib/backend/types';

interface CompletionGateStatusProps {
  decision: CompletionDecision | string;
  orchestratorAction?: string;
}

const GATE_META: Record<string, { label: string; hint: string; className: string; Icon: any }> = {
  ALLOW_COMPLETION: {
    label: 'Completion allowed',
    hint: 'Independent verification passed. The orchestrator may mark the task complete.',
    className: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    Icon: LockOpen,
  },
  REQUIRE_REPAIR: {
    label: 'Repair required',
    hint: 'Verification failed on repairable checks. A bounded repair cycle may run.',
    className: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
    Icon: Wrench,
  },
  REQUIRE_USER: {
    label: 'User review required',
    hint: 'Verification cannot proceed automatically. Repair budget exhausted or input needed.',
    className: 'bg-blue-500/10 text-blue-400 border-blue-500/20',
    Icon: UserCheck,
  },
  BLOCKED: {
    label: 'Completion blocked',
    hint: 'A blocking or fail-closed check prevents completion. Model claims cannot override this.',
    className: 'bg-red-500/10 text-red-400 border-red-500/20',
    Icon: OctagonX,
  },
};

export const CompletionGateStatus: React.FC<CompletionGateStatusProps> = ({
  decision,
  orchestratorAction,
}) => {
  const meta = GATE_META[String(decision)] ?? GATE_META.BLOCKED;
  const { Icon } = meta;
  return (
    <div className={`p-4 rounded-lg border space-y-1 ${meta.className} bg-opacity-40`}>
      <div className="flex items-center justify-between gap-2">
        <span className="flex items-center gap-1.5 text-xs font-bold">
          <Icon className="w-4 h-4" />
          Completion gate: {meta.label}
        </span>
        {orchestratorAction && (
          <span className="text-[10px] font-mono uppercase opacity-80">
            next: {orchestratorAction}
          </span>
        )}
      </div>
      <p className="text-[11px] opacity-90 pl-5">{meta.hint}</p>
    </div>
  );
};
