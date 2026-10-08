'use client';

import React from 'react';
import { ShieldCheck, ShieldAlert, Lock, HardDrive } from 'lucide-react';
import { WorkspaceSecurityStatus } from '@/lib/backend/types';

interface Props {
  status?: WorkspaceSecurityStatus | null;
  readOnly?: boolean;
}

export const FilesystemSecurityBadge: React.FC<Props> = ({ status, readOnly }) => {
  const isReadOnly = readOnly || status?.policy?.read_only;

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 rounded-md border border-slate-700 bg-slate-900 text-xs font-medium text-slate-200">
      <div className="flex items-center gap-1.5">
        <ShieldCheck className="w-4 h-4 text-emerald-400" />
        <span className="text-slate-300">Sandbox Boundary:</span>
        <span className="text-emerald-400 font-semibold">ENFORCED</span>
      </div>

      {isReadOnly && (
        <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 border border-amber-500/30">
          <Lock className="w-3 h-3" /> Read-Only Mode
        </span>
      )}

      {status?.classification && (
        <span className="px-2 py-0.5 rounded bg-blue-500/20 text-blue-300 border border-blue-500/30 font-mono text-[10px]">
          {status.classification}
        </span>
      )}

      {status?.root_path && (
        <span className="flex items-center gap-1 text-slate-400 font-mono text-[11px] truncate max-w-[200px]" title={status.root_path}>
          <HardDrive className="w-3 h-3 text-slate-500" />
          {status.root_path}
        </span>
      )}
    </div>
  );
};
