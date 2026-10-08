'use client';

import React from 'react';
import Link from 'next/link';
import {
  LocalCompatibilityReport,
  LocalModelRecord,
  LocalModelState,
  ResourceEstimate,
} from '@/lib/backend/types';
import { LocalRuntimeHealthBadge } from './LocalRuntimeCard';

function stateStyle(state: LocalModelState): string {
  switch (state) {
    case 'available':
    case 'loaded':
    case 'running':
      return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
    case 'loading':
    case 'unloading':
      return 'bg-blue-500/10 text-blue-400 border-blue-500/20';
    case 'discovered':
    case 'registered':
      return 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20';
    case 'unavailable':
    case 'failed':
      return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
    case 'disabled':
      return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
    default:
      return 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20';
  }
}

export function LocalModelStateBadge({ state }: { state: LocalModelState }) {
  return (
    <span className={`text-xs px-2.5 py-1 rounded-full font-mono border ${stateStyle(state)}`}>
      {state.toUpperCase()}
    </span>
  );
}

interface LocalModelCardProps {
  model: LocalModelRecord;
  onEnable?: (id: string) => void;
  onDisable?: (id: string) => void;
  busy?: boolean;
  linkDetail?: boolean;
}

export function LocalModelCard({ model, onEnable, onDisable, busy = false, linkDetail = true }: LocalModelCardProps) {
  return (
    <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 space-y-3">
      <div className="flex items-start justify-between gap-3">
        <div className="space-y-1">
          <div className="flex items-center gap-2 flex-wrap">
            {linkDetail ? (
              <Link
                href={`/settings/local-models/${encodeURIComponent(model.id)}`}
                className="font-semibold text-sm text-zinc-100 hover:text-blue-400 transition"
              >
                {model.display_name}
              </Link>
            ) : (
              <span className="font-semibold text-sm text-zinc-100">{model.display_name}</span>
            )}
            <LocalModelStateBadge state={model.state} />
            <LocalRuntimeHealthBadge health={model.health} />
          </div>
          <div className="text-xs font-mono text-zinc-500">{model.registry_model_id}</div>
          <div className="text-xs text-zinc-400 flex flex-wrap gap-x-4 gap-y-1">
            <span>
              Context: <strong className="text-zinc-300">{model.context_window?.toLocaleString() ?? 'Unknown'}</strong>
            </span>
            {model.quantization && (
              <span>
                Quant: <strong className="text-zinc-300">{model.quantization}</strong>
              </span>
            )}
            {model.parameter_count_billions !== null && model.parameter_count_billions !== undefined && (
              <span>
                Params: <strong className="text-zinc-300">{model.parameter_count_billions}B</strong>
              </span>
            )}
            {model.requirements_estimated && (
              <span className="text-amber-400">Requirements estimated</span>
            )}
          </div>
          {model.last_error && (
            <div className="text-xs text-rose-400/90 max-w-xl break-words">Last error: {model.last_error}</div>
          )}
        </div>
        <div className="flex gap-2 shrink-0">
          {onEnable && (model.state === 'disabled' || model.state === 'unavailable' || model.state === 'failed') && (
            <button
              disabled={busy}
              onClick={() => onEnable(model.id)}
              className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 disabled:opacity-50 text-emerald-400 border border-emerald-500/20 transition"
            >
              Enable
            </button>
          )}
          {onDisable && model.state !== 'disabled' && (
            <button
              disabled={busy}
              onClick={() => onDisable(model.id)}
              className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-200 border border-zinc-700 transition"
            >
              Disable
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export function ModelResourceEstimate({ estimate }: { estimate: ResourceEstimate }) {
  const fmt = (v?: number | null) => (v === null || v === undefined ? 'Unknown' : `${v.toLocaleString()} MB`);
  return (
    <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 space-y-3">
      <div className="flex items-center justify-between">
        <h4 className="text-xs font-semibold text-zinc-300 uppercase tracking-wider">Resource Estimate</h4>
        <span
          className={`text-xs px-2 py-0.5 rounded font-mono border ${
            estimate.is_estimate
              ? 'bg-amber-500/10 text-amber-400 border-amber-500/20'
              : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
          }`}
        >
          {estimate.is_estimate ? `ESTIMATE (${estimate.confidence})` : 'RUNTIME-REPORTED'}
        </span>
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
        <div>
          <span className="text-zinc-500 block">Est. RAM</span>
          <span className="font-semibold text-zinc-200">{fmt(estimate.estimated_ram_mb)}</span>
        </div>
        <div>
          <span className="text-zinc-500 block">Est. VRAM</span>
          <span className="font-semibold text-zinc-200">{fmt(estimate.estimated_vram_mb)}</span>
        </div>
        <div>
          <span className="text-zinc-500 block">KV Cache</span>
          <span className="font-semibold text-zinc-200">{fmt(estimate.estimated_kv_cache_mb)}</span>
        </div>
        <div>
          <span className="text-zinc-500 block">Runtime Overhead</span>
          <span className="font-semibold text-zinc-200">{estimate.runtime_overhead_mb.toLocaleString()} MB</span>
        </div>
      </div>
      {estimate.assumptions.length > 0 && (
        <ul className="text-xs text-zinc-500 space-y-1 list-disc list-inside">
          {estimate.assumptions.map((a, i) => (
            <li key={i}>{a}</li>
          ))}
        </ul>
      )}
      {estimate.warnings.length > 0 && (
        <ul className="text-xs text-amber-400/90 space-y-1 list-disc list-inside">
          {estimate.warnings.map((w, i) => (
            <li key={i}>{w}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function LocalModelCompatibilityPanel({ report }: { report: LocalCompatibilityReport }) {
  const hwColor =
    report.hardware.status === 'compatible'
      ? 'text-emerald-400'
      : report.hardware.status === 'compatible_with_warnings'
        ? 'text-amber-400'
        : report.hardware.status === 'incompatible'
          ? 'text-rose-400'
          : 'text-zinc-400';
  return (
    <div className="p-5 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-zinc-200">Compatibility Verdict</h3>
        <span className={`text-sm font-bold font-mono ${hwColor}`}>
          {report.hardware.status.toUpperCase().replace(/_/g, ' ')}
        </span>
      </div>
      <div className={`p-3 rounded-lg text-xs border ${report.routable ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-300' : 'bg-rose-500/10 border-rose-500/20 text-rose-300'}`}>
        {report.routable
          ? 'Routable: hardware and context budgets both permit local execution.'
          : 'Not routable: see reasons below. Unknown requirements never auto-pass.'}
      </div>
      <div className="text-xs space-y-1">
        <div className="text-zinc-400">
          Context:{' '}
          <span className={report.context_ok ? 'text-emerald-400' : 'text-rose-400'}>
            {report.context_detail}
          </span>
        </div>
      </div>
      {report.reasons.length > 0 && (
        <ul className="text-xs text-zinc-400 space-y-1 list-disc list-inside max-h-48 overflow-y-auto">
          {report.reasons.map((r, i) => (
            <li key={i} className="break-words">{r}</li>
          ))}
        </ul>
      )}
      <ModelResourceEstimate estimate={report.resource_estimate} />
    </div>
  );
}
