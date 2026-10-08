'use client';

import React from 'react';
import { ShieldCheck, ShieldAlert, AlertTriangle, HelpCircle, FileText, CheckCircle2, XCircle } from 'lucide-react';
import { FilesystemDecision } from '@/lib/backend/types';

interface Props {
  decision?: FilesystemDecision | null;
}

export const FileDecisionInspector: React.FC<Props> = ({ decision }) => {
  if (!decision) {
    return (
      <div className="p-4 rounded-lg border border-slate-800 bg-slate-900/40 text-center text-xs text-slate-500">
        No filesystem evaluation selected for inspection.
      </div>
    );
  }

  const getBadge = () => {
    switch (decision.disposition) {
      case 'allow':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 text-xs font-semibold">
            <CheckCircle2 className="w-3.5 h-3.5" /> ALLOWED
          </span>
        );
      case 'block':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-rose-500/20 text-rose-300 border border-rose-500/30 text-xs font-semibold">
            <XCircle className="w-3.5 h-3.5" /> BLOCKED
          </span>
        );
      case 'require_confirmation':
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/30 text-xs font-semibold">
            <AlertTriangle className="w-3.5 h-3.5" /> CONFIRMATION REQUIRED
          </span>
        );
      default:
        return (
          <span className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-500/20 text-slate-300 border border-slate-500/30 text-xs font-semibold">
            <HelpCircle className="w-3.5 h-3.5" /> UNKNOWN (BLOCKED)
          </span>
        );
    }
  };

  return (
    <div className="space-y-3 p-4 rounded-xl border border-slate-800 bg-slate-900/90 text-xs">
      <div className="flex items-center justify-between pb-2 border-b border-slate-800">
        <div className="flex items-center gap-2">
          <FileText className="w-4 h-4 text-blue-400" />
          <span className="font-semibold text-slate-200">Filesystem Policy Inspection</span>
        </div>
        {getBadge()}
      </div>

      <div className="grid grid-cols-2 gap-3 text-slate-300">
        <div>
          <span className="text-slate-500 text-[11px] block">Operation</span>
          <span className="font-mono font-semibold uppercase text-slate-200">{decision.operation}</span>
        </div>
        <div>
          <span className="text-slate-500 text-[11px] block">Risk Level</span>
          <span className={`font-mono font-semibold uppercase ${
            decision.risk_level === 'critical' || decision.risk_level === 'high' ? 'text-rose-400' : 'text-emerald-400'
          }`}>{decision.risk_level}</span>
        </div>
        <div className="col-span-2">
          <span className="text-slate-500 text-[11px] block">Requested Path</span>
          <span className="font-mono text-slate-100 bg-slate-950 p-1.5 rounded border border-slate-800 block truncate">{decision.requested_path}</span>
        </div>
        {decision.resolved_path && (
          <div className="col-span-2">
            <span className="text-slate-500 text-[11px] block">Resolved Path</span>
            <span className="font-mono text-slate-300 bg-slate-950/60 p-1.5 rounded border border-slate-800 block truncate">{decision.resolved_path}</span>
          </div>
        )}
        <div className="col-span-2">
          <span className="text-slate-500 text-[11px] block">Security Reason</span>
          <p className="text-slate-300 mt-0.5">{decision.reason}</p>
        </div>
        <div>
          <span className="text-slate-500 text-[11px] block">Policy Source</span>
          <span className="font-mono text-slate-400">{decision.policy_source}</span>
        </div>
        <div>
          <span className="text-slate-500 text-[11px] block">Data Classification</span>
          <span className="font-mono text-blue-400">{decision.classification}</span>
        </div>
      </div>

      {decision.warnings && decision.warnings.length > 0 && (
        <div className="mt-2 p-2 rounded bg-amber-950/30 border border-amber-800/40 text-amber-300 space-y-1">
          <span className="font-semibold block text-[11px]">Security Warnings:</span>
          {decision.warnings.map((w, i) => (
            <div key={i} className="text-[11px]">• {w}</div>
          ))}
        </div>
      )}
    </div>
  );
};
