'use client';

import React from 'react';
import { ShieldCheck, ShieldAlert, AlertTriangle, FileText, CheckCircle2, XCircle, Info } from 'lucide-react';
import { PrivacyDecision } from '../lib/backend/types';

interface PrivacyDecisionInspectorProps {
  decision: PrivacyDecision | null;
  onClose?: () => void;
}

export function PrivacyDecisionInspector({ decision, onClose }: PrivacyDecisionInspectorProps) {
  if (!decision) return null;

  const isAllowed = decision.decision === 'ALLOW_LOCAL' || decision.decision === 'ALLOW_ON_PREMISE' || decision.decision === 'ALLOW_ONLINE';

  return (
    <div className="bg-white/95 backdrop-blur-2xl border border-slate-200/90 rounded-3xl p-6 shadow-xl space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-slate-200/80 pb-3">
        <div className="flex items-center space-x-2.5">
          {isAllowed ? (
            <ShieldCheck className="w-5 h-5 text-emerald-600" />
          ) : (
            <ShieldAlert className="w-5 h-5 text-rose-600" />
          )}
          <div>
            <h3 className="text-sm font-extrabold text-slate-900">Privacy Decision Inspector</h3>
            <span className="text-[10px] font-mono text-slate-400">ID: {decision.id}</span>
          </div>
        </div>
        <span className={`px-3 py-1 rounded-full text-xs font-bold font-mono ${
          isAllowed ? 'bg-emerald-100 text-emerald-800 border border-emerald-300' : 'bg-rose-100 text-rose-800 border border-rose-300'
        }`}>
          {decision.decision}
        </span>
      </div>

      {/* Grid details */}
      <div className="grid grid-cols-2 gap-3 text-xs">
        <div className="p-3 bg-slate-100/70 rounded-2xl space-y-1">
          <span className="text-[10px] font-extrabold uppercase text-slate-400">Classification</span>
          <div className="font-bold text-slate-900 font-mono">{decision.classification}</div>
        </div>
        <div className="p-3 bg-slate-100/70 rounded-2xl space-y-1">
          <span className="text-[10px] font-extrabold uppercase text-slate-400">Requested Mode</span>
          <div className="font-bold text-slate-900 font-mono">{decision.requested_execution_mode}</div>
        </div>
        <div className="p-3 bg-slate-100/70 rounded-2xl space-y-1">
          <span className="text-[10px] font-extrabold uppercase text-slate-400">Policy Source</span>
          <div className="font-bold text-slate-900 font-mono">{decision.selected_policy_source} (v{decision.policy_version})</div>
        </div>
        <div className="p-3 bg-slate-100/70 rounded-2xl space-y-1">
          <span className="text-[10px] font-extrabold uppercase text-slate-400">Confidence</span>
          <div className="font-bold text-slate-900 font-mono">{decision.confidence}</div>
        </div>
      </div>

      {/* Allowed vs Blocked Modes */}
      <div className="p-3.5 bg-slate-50 border border-slate-200/80 rounded-2xl space-y-2 text-xs">
        <span className="text-[10px] font-extrabold uppercase text-slate-400">Allowed Execution Modes</span>
        <div className="flex flex-wrap gap-1.5">
          {decision.allowed_execution_modes.map(mode => (
            <span key={mode} className="px-2.5 py-1 rounded-xl bg-emerald-100 border border-emerald-300 text-emerald-800 font-mono font-bold text-[11px] flex items-center space-x-1">
              <CheckCircle2 className="w-3 h-3 text-emerald-600" />
              <span>{mode}</span>
            </span>
          ))}
          {!decision.allowed_execution_modes.includes('cloud' as any) && (
            <span className="px-2.5 py-1 rounded-xl bg-rose-100 border border-rose-300 text-rose-800 font-mono font-bold text-[11px] flex items-center space-x-1 opacity-70">
              <XCircle className="w-3 h-3 text-rose-600" />
              <span>CLOUD (BLOCKED)</span>
            </span>
          )}
        </div>
      </div>

      {/* Reason */}
      <div className="p-3.5 bg-amber-50/80 border border-amber-200 rounded-2xl text-xs space-y-1 text-amber-900">
        <div className="flex items-center space-x-1.5 font-bold">
          <Info className="w-4 h-4 text-amber-600" />
          <span>Policy Explanation</span>
        </div>
        <p className="font-medium text-[11px] leading-relaxed pl-5">{decision.reason}</p>
      </div>

      {onClose && (
        <div className="flex justify-end pt-2">
          <button onClick={onClose} className="px-4 py-2 rounded-xl text-xs font-bold bg-slate-900 text-white hover:bg-slate-800 transition-all">
            Close Inspector
          </button>
        </div>
      )}
    </div>
  );
}
