'use client';

import React from 'react';
import { AlertTriangle } from 'lucide-react';

interface VerificationWarningProps {
  warnings: string[];
}

export const VerificationWarning: React.FC<VerificationWarningProps> = ({ warnings }) => {
  if (!warnings || warnings.length === 0) return null;
  return (
    <div className="p-4 rounded-lg border border-amber-500/20 bg-amber-500/5 space-y-2">
      <div className="flex items-center gap-1.5 text-xs font-bold text-amber-400">
        <AlertTriangle className="w-3.5 h-3.5" />
        Warnings ({warnings.length})
      </div>
      <ul className="space-y-1 pl-5 list-disc">
        {warnings.map((warning, idx) => (
          <li key={idx} className="text-[11px] text-amber-200/80">
            {warning}
          </li>
        ))}
      </ul>
    </div>
  );
};
