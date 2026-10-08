'use client';

import React from 'react';
import { ToolDescriptor, ToolDecision } from '@/lib/backend/types';
import { ToolSecurityBadge } from './ToolSecurityBadge';

interface ToolPermissionInspectorProps {
  tool: ToolDescriptor | null;
  decision?: ToolDecision | null;
}

export const ToolPermissionInspector: React.FC<ToolPermissionInspectorProps> = ({
  tool,
  decision,
}) => {
  if (!tool) {
    return (
      <div className="p-6 bg-zinc-900 border border-zinc-800 rounded-lg text-center text-zinc-500 text-sm">
        Select a tool from the registry to inspect capability policies and security rules.
      </div>
    );
  }

  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden space-y-4 p-5">
      <div className="flex items-start justify-between border-b border-zinc-800 pb-4">
        <div>
          <div className="flex items-center space-x-2">
            <h4 className="text-base font-semibold text-zinc-100">{tool.name}</h4>
            <span className="text-xs font-mono text-zinc-500">v{tool.version}</span>
          </div>
          <p className="text-xs text-zinc-400 mt-1">{tool.description}</p>
        </div>
        <ToolSecurityBadge
          status={tool.enabled ? 'ALLOWED' : 'BLOCKED'}
          riskLevel={tool.risk_level}
        />
      </div>

      <div className="grid grid-cols-2 gap-4 text-xs">
        <div>
          <span className="text-zinc-500 font-mono uppercase tracking-wider block mb-1">Source</span>
          <span className="px-2 py-0.5 bg-zinc-800 text-zinc-300 font-mono rounded">{tool.source}</span>
        </div>
        <div>
          <span className="text-zinc-500 font-mono uppercase tracking-wider block mb-1">Timeout</span>
          <span className="font-mono text-zinc-300">{tool.timeout_ms} ms</span>
        </div>
      </div>

      <div>
        <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider block mb-2">
          Declared Capabilities
        </span>
        <div className="flex flex-wrap gap-1.5">
          {tool.capabilities.map((cap) => (
            <span
              key={cap}
              className="px-2 py-1 bg-zinc-950 border border-zinc-800 text-emerald-400 text-xs font-mono rounded"
            >
              {cap}
            </span>
          ))}
        </div>
      </div>

      {decision && (
        <div className="mt-4 pt-4 border-t border-zinc-800 space-y-2">
          <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider block">
            Latest Policy Evaluation Result
          </span>
          <div className="p-3 bg-zinc-950 border border-zinc-800 rounded space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-zinc-300">{decision.policy_source}</span>
              <ToolSecurityBadge status={decision.decision} riskLevel={decision.risk_level} />
            </div>
            <p className="text-xs text-zinc-400">{decision.reason}</p>
          </div>
        </div>
      )}

      <div>
        <span className="text-zinc-500 font-mono text-xs uppercase tracking-wider block mb-1">
          Input Argument Schema
        </span>
        <pre className="p-3 bg-zinc-950 border border-zinc-800 rounded text-xs font-mono text-zinc-300 overflow-x-auto max-h-48">
          {JSON.stringify(tool.input_schema, null, 2)}
        </pre>
      </div>
    </div>
  );
};
