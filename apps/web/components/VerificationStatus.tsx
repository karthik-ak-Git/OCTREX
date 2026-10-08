'use client';

import React from 'react';
import { ShieldCheck, ShieldAlert, ShieldX, ShieldQuestion, Ban } from 'lucide-react';
import type { VerificationStatus as VerificationStatusType } from '@/lib/backend/types';

interface VerificationStatusProps {
  status: VerificationStatusType | string;
  confidence?: number;
  compact?: boolean;
}

const STATUS_META: Record<string, { label: string; className: string; Icon: any }> = {
  PASS: {
    label: 'Verified',
    className: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    Icon: ShieldCheck,
  },
  PASS_WITH_WARNINGS: {
    label: 'Verified with warnings',
    className: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
    Icon: ShieldAlert,
  },
  FAIL: {
    label: 'Verification failed',
    className: 'bg-rose-500/10 text-rose-400 border-rose-500/20',
    Icon: ShieldX,
  },
  BLOCKED: {
    label: 'Blocked',
    className: 'bg-red-500/10 text-red-400 border-red-500/20',
    Icon: Ban,
  },
  UNKNOWN: {
    label: 'Unknown (not verified)',
    className: 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20',
    Icon: ShieldQuestion,
  },
  NOT_VERIFIED: {
    label: 'Not verified',
    className: 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20',
    Icon: ShieldQuestion,
  },
};

export const VerificationStatus: React.FC<VerificationStatusProps> = ({
  status,
  confidence,
  compact = false,
}) => {
  const meta = STATUS_META[String(status)] ?? STATUS_META.UNKNOWN;
  const { Icon } = meta;
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border font-mono font-semibold ${meta.className} ${
        compact ? 'px-2 py-0.5 text-[10px]' : 'px-2.5 py-1 text-[11px]'
      }`}
      title={
        typeof confidence === 'number'
          ? `Authoritative-evidence confidence: ${(confidence * 100).toFixed(0)}%. Model claims never count as evidence.`
          : 'Model claims never count as evidence.'
      }
    >
      <Icon className={compact ? 'w-3 h-3' : 'w-3.5 h-3.5'} />
      {meta.label}
      {typeof confidence === 'number' && !compact && (
        <span className="opacity-70">{(confidence * 100).toFixed(0)}%</span>
      )}
    </span>
  );
};
