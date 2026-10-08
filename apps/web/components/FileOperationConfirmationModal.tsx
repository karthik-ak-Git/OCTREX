'use client';

import React from 'react';
import { AlertTriangle, Trash2, FileWarning, ShieldAlert } from 'lucide-react';
import { FilesystemDecision } from '@/lib/backend/types';

interface Props {
  isOpen: boolean;
  decision?: FilesystemDecision | null;
  onConfirm: () => void;
  onCancel: () => void;
}

export const FileOperationConfirmationModal: React.FC<Props> = ({
  isOpen,
  decision,
  onConfirm,
  onCancel,
}) => {
  if (!isOpen || !decision) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="w-full max-w-md rounded-xl border border-rose-800/60 bg-slate-900 shadow-2xl p-5 space-y-4">
        <div className="flex items-center gap-3 text-rose-400 border-b border-slate-800 pb-3">
          <ShieldAlert className="w-6 h-6 text-rose-500 shrink-0" />
          <div>
            <h3 className="text-base font-semibold text-slate-100">Confirm High-Risk Filesystem Action</h3>
            <p className="text-xs text-rose-400">Explicit user authorization required</p>
          </div>
        </div>

        <div className="space-y-3 text-xs text-slate-300">
          <p className="leading-relaxed">
            An agent or process is requesting permission to execute a high-risk filesystem operation:
          </p>

          <div className="p-3 rounded-lg bg-slate-950 border border-slate-800 space-y-1.5 font-mono text-[11px]">
            <div className="flex justify-between">
              <span className="text-slate-500">Operation:</span>
              <span className="text-rose-400 font-bold uppercase">{decision.operation}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-500">Target Path:</span>
              <span className="text-slate-100 truncate max-w-[220px]" title={decision.requested_path}>{decision.requested_path}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-500">Risk Level:</span>
              <span className="text-amber-400 font-bold uppercase">{decision.risk_level}</span>
            </div>
          </div>

          <div className="p-2.5 rounded bg-amber-950/30 border border-amber-800/40 text-amber-300 flex items-start gap-2">
            <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <span className="text-[11px] leading-normal">{decision.reason}</span>
          </div>
        </div>

        <div className="flex justify-end gap-2 pt-2 border-t border-slate-800">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-lg border border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700 text-xs font-medium transition-colors"
          >
            Block Action
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 rounded-lg bg-rose-600 text-white hover:bg-rose-500 text-xs font-semibold shadow-lg shadow-rose-600/20 transition-colors flex items-center gap-1.5"
          >
            <Trash2 className="w-3.5 h-3.5" /> Authorize Operation
          </button>
        </div>
      </div>
    </div>
  );
};
