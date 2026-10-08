'use client';

import React from 'react';
import { FileSearch } from 'lucide-react';

interface VerificationEvidenceProps {
  evidenceRefs: string[];
}

/**
 * Evidence references only. Raw secrets, full file contents, and unbounded
 * command output are never exposed in diagnostics.
 */
export const VerificationEvidence: React.FC<VerificationEvidenceProps> = ({ evidenceRefs }) => {
  if (!evidenceRefs || evidenceRefs.length === 0) {
    return (
      <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 text-xs text-zinc-500 text-center">
        No evidence references recorded.
      </div>
    );
  }
  return (
    <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 space-y-2">
      <div className="flex items-center gap-1.5 text-xs font-semibold text-zinc-300">
        <FileSearch className="w-3.5 h-3.5 text-zinc-500" />
        Evidence references ({evidenceRefs.length})
      </div>
      <ul className="space-y-1">
        {evidenceRefs.map((ref) => (
          <li key={ref} className="text-[11px] font-mono text-zinc-500 break-all">
            {ref}
          </li>
        ))}
      </ul>
      <p className="text-[10px] text-zinc-600">
        References carry provenance only. Raw content is never exposed here.
      </p>
    </div>
  );
};
