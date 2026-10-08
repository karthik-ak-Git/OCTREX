'use client';

import React from 'react';
import { Cpu, Layers, Zap, AlertTriangle, CheckCircle } from 'lucide-react';
import { TokenCountKind } from '../lib/backend/types';

interface ContextBudgetIndicatorProps {
  usedTokens: number;
  usableBudget: number;
  contextWindow: number;
  tokenCountKind?: TokenCountKind;
  compacted?: boolean;
  modelId?: string;
  className?: string;
}

export const ContextBudgetIndicator: React.FC<ContextBudgetIndicatorProps> = ({
  usedTokens,
  usableBudget,
  contextWindow,
  tokenCountKind = 'estimated',
  compacted = false,
  modelId,
  className = '',
}) => {
  const safeUsable = usableBudget > 0 ? usableBudget : 8192;
  const utilization = Math.min(100, Math.round((usedTokens / safeUsable) * 100));

  let barColor = 'bg-emerald-500';
  let badgeColor = 'bg-emerald-100 text-emerald-800 border-emerald-300';
  if (utilization >= 90) {
    barColor = 'bg-rose-500';
    badgeColor = 'bg-rose-100 text-rose-800 border-rose-300';
  } else if (utilization >= 75) {
    barColor = 'bg-amber-500';
    badgeColor = 'bg-amber-100 text-amber-800 border-amber-300';
  }

  return (
    <div className={`p-3 bg-white/80 backdrop-blur-md border border-slate-200/90 rounded-2xl shadow-2xs flex items-center justify-between text-xs font-mono select-none ${className}`}>
      <div className="flex items-center space-x-2.5">
        <div className="w-6 h-6 rounded-lg bg-slate-900 text-white flex items-center justify-center font-bold">
          <Cpu className="w-3.5 h-3.5" />
        </div>
        <div>
          <div className="flex items-center space-x-1.5 font-bold text-slate-800">
            <span>Context Budget</span>
            {modelId && <span className="text-[10px] text-slate-400 font-normal">({modelId})</span>}
          </div>
          <div className="text-[10px] text-slate-500 flex items-center space-x-1">
            <span>{usedTokens.toLocaleString()} / {safeUsable.toLocaleString()} tokens</span>
            <span className="text-slate-400">•</span>
            <span className="capitalize">{tokenCountKind}</span>
          </div>
        </div>
      </div>

      <div className="flex items-center space-x-3">
        {compacted && (
          <span className="px-2 py-0.5 rounded-full text-[9px] font-bold bg-purple-100 text-purple-800 border border-purple-300 flex items-center space-x-1">
            <Layers className="w-2.5 h-2.5" />
            <span>COMPACTED</span>
          </span>
        )}

        <div className="w-24 bg-slate-200 h-2 rounded-full overflow-hidden">
          <div
            className={`h-full transition-all duration-300 ${barColor}`}
            style={{ width: `${utilization}%` }}
          />
        </div>

        <span className={`px-2 py-0.5 rounded-full text-[10px] font-extrabold border ${badgeColor}`}>
          {utilization}%
        </span>
      </div>
    </div>
  );
};
