'use client';

import React from 'react';
import { ShieldCheck, AlertTriangle, FolderCheck, CornerDownRight } from 'lucide-react';

interface Props {
  rootPath: string;
  relativePath: string;
  isValid: boolean;
  resolvedPath?: string;
  reason?: string;
}

export const WorkspaceBoundaryIndicator: React.FC<Props> = ({
  rootPath,
  relativePath,
  isValid,
  resolvedPath,
  reason,
}) => {
  return (
    <div className={`p-3 rounded-lg border text-xs font-mono transition-all ${
      isValid
        ? 'border-emerald-800/50 bg-emerald-950/20 text-emerald-300'
        : 'border-rose-800/50 bg-rose-950/20 text-rose-300'
    }`}>
      <div className="flex items-center justify-between font-sans mb-1 font-semibold">
        <div className="flex items-center gap-1.5">
          {isValid ? (
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
          ) : (
            <AlertTriangle className="w-4 h-4 text-rose-400" />
          )}
          <span>{isValid ? 'Path Traversal Boundary Checked' : 'Path Escape Blocked'}</span>
        </div>
        <span className={`px-2 py-0.5 rounded text-[10px] uppercase tracking-wider ${
          isValid ? 'bg-emerald-500/20 border border-emerald-500/30' : 'bg-rose-500/20 border border-rose-500/30'
        }`}>
          {isValid ? 'VALID BOUNDARY' : 'BOUNDARY ESCAPE'}
        </span>
      </div>

      <div className="space-y-1 text-[11px] text-slate-300 mt-2 font-mono">
        <div className="flex items-center gap-1.5 text-slate-400">
          <FolderCheck className="w-3.5 h-3.5 text-slate-500 shrink-0" />
          <span>Root:</span>
          <span className="text-slate-200 truncate">{rootPath || '[No Active Workspace]'}</span>
        </div>
        <div className="flex items-center gap-1.5 text-slate-400">
          <CornerDownRight className="w-3.5 h-3.5 text-slate-500 shrink-0" />
          <span>Requested:</span>
          <span className="text-slate-100 font-semibold">{relativePath}</span>
        </div>
        {resolvedPath && (
          <div className="text-slate-400 text-[10px]">
            Resolved Path: <span className="text-slate-300">{resolvedPath}</span>
          </div>
        )}
        {!isValid && reason && (
          <div className="mt-1.5 text-rose-400 font-sans text-xs bg-rose-900/30 p-1.5 rounded border border-rose-800/40">
            <strong>Block Reason:</strong> {reason}
          </div>
        )}
      </div>
    </div>
  );
};
