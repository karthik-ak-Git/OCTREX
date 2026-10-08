'use client';

import React from 'react';
import { ToolRequest, ToolDecision } from '@/lib/backend/types';
import { ToolSecurityBadge } from './ToolSecurityBadge';

interface ToolConsentModalProps {
  isOpen: boolean;
  request: ToolRequest | null;
  decision: ToolDecision | null;
  onApprove: () => void;
  onDeny: () => void;
}

export const ToolConsentModal: React.FC<ToolConsentModalProps> = ({
  isOpen,
  request,
  decision,
  onApprove,
  onDeny,
}) => {
  if (!isOpen || !request || !decision) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div className="w-full max-w-lg bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl overflow-hidden">
        <div className="px-6 py-4 border-b border-zinc-800 bg-zinc-950 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <ToolSecurityBadge status="REQUIRE_CONSENT" riskLevel={decision.risk_level} />
            <h3 className="text-base font-semibold text-zinc-100">Tool Execution Approval</h3>
          </div>
        </div>

        <div className="p-6 space-y-4 text-sm text-zinc-300">
          <div>
            <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider">Tool ID</span>
            <p className="font-mono text-zinc-100 font-semibold">{request.tool_id}</p>
          </div>

          <div>
            <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider">Reason</span>
            <p className="text-zinc-200 mt-0.5">{decision.reason}</p>
          </div>

          {decision.denied_capabilities.length > 0 && (
            <div>
              <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider">Requested Capabilities</span>
              <div className="flex flex-wrap gap-1.5 mt-1.5">
                {decision.denied_capabilities.map((cap) => (
                  <span
                    key={cap}
                    className="px-2 py-0.5 bg-amber-950/60 border border-amber-700/50 text-amber-300 text-xs font-mono rounded"
                  >
                    {cap}
                  </span>
                ))}
              </div>
            </div>
          )}

          <div>
            <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider">Arguments</span>
            <pre className="mt-1 p-3 bg-zinc-950 border border-zinc-800 rounded text-xs font-mono text-zinc-300 overflow-x-auto max-h-40">
              {JSON.stringify(request.arguments, null, 2)}
            </pre>
          </div>

          <div className="p-3 bg-amber-950/30 border border-amber-800/40 rounded text-xs text-amber-200/90">
            <strong>Security Notice:</strong> Authorizing this execution grants capability only for this invocation. Backend policies will continuously sanitize and validate execution results.
          </div>
        </div>

        <div className="px-6 py-4 border-t border-zinc-800 bg-zinc-950 flex items-center justify-end space-x-3">
          <button
            onClick={onDeny}
            className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded text-xs font-medium transition"
          >
            Deny Execution
          </button>
          <button
            onClick={onApprove}
            className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-medium rounded text-xs transition"
          >
            Approve Execution
          </button>
        </div>
      </div>
    </div>
  );
};
