'use client';

import React from 'react';
import { LocalRuntimeDescriptor, LocalRuntimeHealth } from '@/lib/backend/types';

function healthStyle(health: LocalRuntimeHealth): string {
  switch (health) {
    case 'healthy':
      return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
    case 'degraded':
      return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
    case 'unavailable':
      return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
    case 'unknown':
    default:
      return 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20';
  }
}

export function LocalRuntimeHealthBadge({ health }: { health: LocalRuntimeHealth }) {
  return (
    <span className={`text-xs px-2.5 py-1 rounded-full font-mono border ${healthStyle(health)}`}>
      {health.toUpperCase()}
    </span>
  );
}

interface LocalRuntimeCardProps {
  runtime: LocalRuntimeDescriptor;
  modelCount?: number;
  onRefresh?: (id: string) => void;
  onTest?: (id: string) => void;
  onStart?: (id: string) => void;
  onStop?: (id: string) => void;
  onDiscoverModels?: (id: string) => void;
  busy?: boolean;
}

export function LocalRuntimeCard({
  runtime,
  modelCount,
  onRefresh,
  onTest,
  onStart,
  onStop,
  onDiscoverModels,
  busy = false,
}: LocalRuntimeCardProps) {
  return (
    <div className="p-5 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-sm text-zinc-100">{runtime.name}</span>
            <LocalRuntimeHealthBadge health={runtime.health} />
          </div>
          <div className="text-xs font-mono text-zinc-500">
            {runtime.runtime_type} · {runtime.endpoint}
          </div>
          {runtime.version && (
            <div className="text-xs text-zinc-500">Version: {runtime.version}</div>
          )}
          {typeof modelCount === 'number' && (
            <div className="text-xs text-zinc-400">
              Managed models: <strong className="text-zinc-200">{modelCount}</strong>
            </div>
          )}
          {runtime.last_error && (
            <div className="text-xs text-rose-400/90 max-w-xl break-words">
              Last error: {runtime.last_error}
            </div>
          )}
        </div>
        <span className="text-xs px-2 py-1 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 font-mono">
          {runtime.execution_mode}
        </span>
      </div>

      <div className="flex flex-wrap gap-2">
        {onRefresh && (
          <button
            disabled={busy}
            onClick={() => onRefresh(runtime.id)}
            className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-200 border border-zinc-700 transition"
          >
            Refresh Health
          </button>
        )}
        {onTest && (
          <button
            disabled={busy}
            onClick={() => onTest(runtime.id)}
            className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-200 border border-zinc-700 transition"
          >
            Test Connection
          </button>
        )}
        {onDiscoverModels && (
          <button
            disabled={busy}
            onClick={() => onDiscoverModels(runtime.id)}
            className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-blue-500/10 hover:bg-blue-500/20 disabled:opacity-50 text-blue-400 border border-blue-500/20 transition"
          >
            Discover Models
          </button>
        )}
        {onStart && (
          <button
            disabled={busy}
            onClick={() => onStart(runtime.id)}
            className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 disabled:opacity-50 text-emerald-400 border border-emerald-500/20 transition"
          >
            Start
          </button>
        )}
        {onStop && (
          <button
            disabled={busy}
            onClick={() => onStop(runtime.id)}
            className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-rose-500/10 hover:bg-rose-500/20 disabled:opacity-50 text-rose-400 border border-rose-500/20 transition"
          >
            Stop
          </button>
        )}
      </div>
    </div>
  );
}
