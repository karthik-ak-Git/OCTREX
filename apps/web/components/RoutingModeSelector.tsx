'use client';

import React from 'react';
import { RoutingMode } from '@/lib/backend/types';

interface RoutingModeSelectorProps {
  value: RoutingMode;
  onChange: (mode: RoutingMode) => void;
  disabled?: boolean;
}

const MODES: { id: RoutingMode; label: string; description: string }[] = [
  {
    id: 'auto',
    label: 'Auto (Recommended)',
    description: 'Selects the highest-scoring permitted model governed by policy and hardware capabilities.',
  },
  {
    id: 'local_only',
    label: 'Local Only',
    description: 'Enforces local model execution only. Third-party cloud calls are prohibited.',
  },
  {
    id: 'on_premise_only',
    label: 'On-Premise Only',
    description: 'Restricts model execution to trusted organization infrastructure.',
  },
  {
    id: 'online_only',
    label: 'Online Only',
    description: 'Requires third-party cloud model execution (subject to privacy gate approval).',
  },
];

export function RoutingModeSelector({
  value,
  onChange,
  disabled = false,
}: RoutingModeSelectorProps) {
  return (
    <div className="space-y-3">
      <label className="block text-sm font-semibold text-zinc-200">
        Model Execution Routing Mode
      </label>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {MODES.map((mode) => {
          const isSelected = value === mode.id;
          return (
            <button
              key={mode.id}
              type="button"
              disabled={disabled}
              onClick={() => onChange(mode.id)}
              className={`text-left p-3.5 rounded-lg border transition-all ${
                isSelected
                  ? 'bg-blue-500/10 border-blue-500/40 text-blue-300 shadow-sm'
                  : 'bg-zinc-900/60 border-zinc-800/60 text-zinc-400 hover:border-zinc-700 hover:bg-zinc-900'
              } ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
            >
              <div className="flex items-center justify-between mb-1">
                <span className="text-sm font-medium text-zinc-100">{mode.label}</span>
                {isSelected && (
                  <span className="w-2 h-2 rounded-full bg-blue-400 ring-4 ring-blue-400/20" />
                )}
              </div>
              <p className="text-xs text-zinc-400 leading-relaxed">{mode.description}</p>
            </button>
          );
        })}
      </div>
    </div>
  );
}
