'use client';

import React from 'react';
import { XOctagon } from 'lucide-react';

interface VerificationFailureProps {
  failures: string[];
}

export const VerificationFailure: React.FC<VerificationFailureProps> = ({ failures }) => {
  if (!failures || failures.length === 0) return null;
  return (
    <div className="p-4 rounded-lg border border-rose-500/20 bg-rose-500/5 space-y-2">
      <div className="flex items-center gap-1.5 text-xs font-bold text-rose-400">
        <XOctagon className="w-3.5 h-3.5" />
        Failures ({failures.length})
      </div>
      <ul className="space-y-1 pl-5 list-disc">
        {failures.map((failure, idx) => (
          <li key={idx} className="text-[11px] text-rose-200/80 font-mono">
            {failure}
          </li>
        ))}
      </ul>
      <p className="text-[10px] text-zinc-500 pl-5">
        A model stating “done” does not resolve these. Each failure needs authoritative evidence.
      </p>
    </div>
  );
};
