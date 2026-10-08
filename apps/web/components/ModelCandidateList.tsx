'use client';

import React from 'react';
import { ModelDescriptor } from '@/lib/backend/types';
import { ModelCompatibilityBadge } from './ModelCompatibilityBadge';

interface ModelCandidateListProps {
  models: ModelDescriptor[];
  selectedModelId?: string;
  onSelectModel?: (modelId: string) => void;
  isLoading?: boolean;
}

export function ModelCandidateList({
  models,
  selectedModelId,
  onSelectModel,
  isLoading = false,
}: ModelCandidateListProps) {
  if (isLoading) {
    return (
      <div className="p-8 text-center text-zinc-500 text-sm animate-pulse">
        Evaluating candidate models and hardware profiles...
      </div>
    );
  }

  if (models.length === 0) {
    return (
      <div className="p-6 text-center border border-dashed border-zinc-800 rounded-lg text-zinc-500 text-sm">
        No candidate models available matching current constraints.
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {models.map((model) => {
        const isSelected = selectedModelId === model.id;
        return (
          <div
            key={model.id}
            onClick={() => onSelectModel?.(model.id)}
            className={`p-3.5 rounded-lg border transition-all flex items-center justify-between cursor-pointer ${
              isSelected
                ? 'bg-blue-500/10 border-blue-500/40 text-blue-200'
                : 'bg-zinc-900/60 border-zinc-800/60 hover:border-zinc-700 text-zinc-300'
            }`}
          >
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <span className="font-semibold text-sm text-zinc-100">{model.display_name}</span>
                <span className="text-xs font-mono text-zinc-500">({model.id})</span>
                <ModelCompatibilityBadge executionMode={model.execution_mode} />
              </div>
              <div className="text-xs text-zinc-400 flex items-center gap-3">
                <span>Provider: <strong className="text-zinc-300">{model.provider_id}</strong></span>
                <span>Context: <strong className="text-zinc-300">{model.context_window?.toLocaleString() || 'N/A'} tokens</strong></span>
                <span>Output: <strong className="text-zinc-300">{model.max_output_tokens?.toLocaleString() || 'N/A'} tokens</strong></span>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-xs px-2 py-1 rounded bg-zinc-800 text-zinc-300 font-mono">
                {model.execution_mode}
              </span>
              {isSelected && (
                <span className="text-xs px-2 py-0.5 rounded bg-blue-500 text-white font-medium">
                  Active Choice
                </span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
