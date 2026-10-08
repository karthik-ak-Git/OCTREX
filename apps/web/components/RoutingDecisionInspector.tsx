'use client';

import React from 'react';
import { RoutingDecision } from '@/lib/backend/types';
import { ModelCompatibilityBadge } from './ModelCompatibilityBadge';

interface RoutingDecisionInspectorProps {
  decision: RoutingDecision | null;
  isOpen: boolean;
  onClose: () => void;
}

export function RoutingDecisionInspector({
  decision,
  isOpen,
  onClose,
}: RoutingDecisionInspectorProps) {
  if (!isOpen || !decision) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl max-w-3xl w-full max-h-[85vh] overflow-hidden shadow-2xl flex flex-col">
        {/* Header */}
        <div className="p-4 border-b border-zinc-800 flex items-center justify-between bg-zinc-950/50">
          <div>
            <h3 className="text-base font-semibold text-zinc-100 flex items-center gap-2">
              <span>Model Routing Decision Inspector</span>
              <span className="text-xs px-2 py-0.5 rounded font-mono bg-zinc-800 text-zinc-300">
                {decision.id}
              </span>
            </h3>
            <p className="text-xs text-zinc-400 mt-0.5">
              Authorization-aware model selection evidence and security audit trail
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-400 hover:text-zinc-200 text-sm px-2.5 py-1 rounded-md bg-zinc-800 hover:bg-zinc-700 transition"
          >
            Close
          </button>
        </div>

        {/* Content */}
        <div className="p-5 overflow-y-auto space-y-6 text-sm">
          {/* Summary Box */}
          <div className="p-4 rounded-lg border bg-zinc-950/40 border-zinc-800 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                Decision Outcome
              </span>
              <span className="text-xs font-mono text-zinc-400">
                State: <strong className="text-zinc-200">{decision.state}</strong>
              </span>
            </div>
            <div className="text-sm font-medium text-zinc-100 flex items-center gap-2">
              <span>Selected Model:</span>
              <span className="font-mono text-blue-400">
                {decision.selected_model_id || 'None (Rejected)'}
              </span>
              {decision.execution_mode && (
                <ModelCompatibilityBadge executionMode={decision.execution_mode} />
              )}
            </div>
            <p className="text-xs text-zinc-300 bg-zinc-900/60 p-2.5 rounded border border-zinc-800/60 font-mono">
              {decision.reason}
            </p>
          </div>

          {/* Evidence Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-3.5 rounded-lg border border-zinc-800 bg-zinc-900/40">
              <h4 className="text-xs font-semibold text-zinc-300 mb-1.5 uppercase">
                Privacy Policy Status
              </h4>
              <p className="text-xs text-zinc-400">{decision.evidence.privacy_status}</p>
            </div>
            <div className="p-3.5 rounded-lg border border-zinc-800 bg-zinc-900/40">
              <h4 className="text-xs font-semibold text-zinc-300 mb-1.5 uppercase">
                Hardware Evaluation
              </h4>
              <p className="text-xs text-zinc-400">{decision.evidence.hardware_status}</p>
            </div>
            <div className="p-3.5 rounded-lg border border-zinc-800 bg-zinc-900/40">
              <h4 className="text-xs font-semibold text-zinc-300 mb-1.5 uppercase">
                Context Window Bounds
              </h4>
              <p className="text-xs text-zinc-400">{decision.evidence.context_status}</p>
            </div>
            <div className="p-3.5 rounded-lg border border-zinc-800 bg-zinc-900/40">
              <h4 className="text-xs font-semibold text-zinc-300 mb-1.5 uppercase">
                Provider Health Status
              </h4>
              <p className="text-xs text-zinc-400">{decision.evidence.provider_status}</p>
            </div>
          </div>

          {/* Candidate Models Evaluated */}
          {decision.evidence.excluded_candidates.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold text-zinc-300 mb-2 uppercase">
                Eliminated Candidate Models ({decision.evidence.excluded_candidates.length})
              </h4>
              <div className="space-y-2 max-h-48 overflow-y-auto pr-1">
                {decision.evidence.excluded_candidates.map((cand) => (
                  <div
                    key={cand.model_id}
                    className="p-3 rounded border border-zinc-800/80 bg-zinc-950/40 flex items-start justify-between text-xs"
                  >
                    <div>
                      <div className="font-semibold text-zinc-200">{cand.display_name}</div>
                      <div className="text-zinc-500 font-mono text-[11px]">
                        ID: {cand.model_id} | Provider: {cand.provider_id} | Mode: {cand.execution_mode}
                      </div>
                      <div className="text-rose-400/90 text-[11px] mt-1 font-mono">
                        {cand.details}
                      </div>
                    </div>
                    <span className="px-2 py-0.5 rounded text-[10px] font-semibold bg-rose-500/10 text-rose-400 border border-rose-500/20">
                      Excluded
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Policy Evidence Trace */}
          <div>
            <h4 className="text-xs font-semibold text-zinc-300 mb-2 uppercase">
              Authoritative Policy Evidence Trace
            </h4>
            <ul className="space-y-1 bg-zinc-950/60 p-3 rounded-lg border border-zinc-800 text-xs font-mono text-zinc-400">
              {decision.policy_evidence.map((item, idx) => (
                <li key={idx} className="flex items-center gap-2">
                  <span className="text-blue-400">›</span>
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
