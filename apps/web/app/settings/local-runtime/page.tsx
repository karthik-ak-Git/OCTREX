'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { backendClient } from '@/lib/backend/client';
import { LocalInferenceMetrics, LocalRuntimeDescriptor } from '@/lib/backend/types';
import { LocalRuntimeCard } from '@/components/LocalRuntimeCard';

export default function LocalRuntimePage() {
  const [runtimes, setRuntimes] = useState<LocalRuntimeDescriptor[]>([]);
  const [modelCounts, setModelCounts] = useState<Record<string, number>>({});
  const [metrics, setMetrics] = useState<LocalInferenceMetrics[]>([]);
  const [activeCalls, setActiveCalls] = useState<string[]>([]);
  const [slotOccupants, setSlotOccupants] = useState<string[]>([]);
  const [modelDir, setModelDir] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const [newEndpoint, setNewEndpoint] = useState('http://127.0.0.1:11434');
  const [newType, setNewType] = useState('ollama');
  const [newName, setNewName] = useState('');

  useEffect(() => {
    loadAll();
  }, []);

  const loadAll = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [rt, models, met, storage] = await Promise.all([
        backendClient.listLocalRuntimes(),
        backendClient.listLocalModels().catch(() => ({ models: [] as never[] })),
        backendClient.listLocalInferenceMetrics().catch(() => null),
        backendClient.getLocalStorage().catch(() => null),
      ]);
      setRuntimes(rt.runtimes || []);
      const counts: Record<string, number> = {};
      for (const m of (models as { models: { runtime_id: string }[] }).models || []) {
        counts[m.runtime_id] = (counts[m.runtime_id] || 0) + 1;
      }
      setModelCounts(counts);
      if (met) {
        setMetrics(met.metrics || []);
        setActiveCalls(met.active_calls || []);
        setSlotOccupants(met.slot_occupants || []);
      }
      if (storage) setModelDir(storage.model_dir || '');
    } catch (e: any) {
      setError(e.message || 'Failed to load local runtimes');
    } finally {
      setIsLoading(false);
    }
  };

  const runAction = async (label: string, fn: () => Promise<{ success: boolean; error?: string } | unknown>) => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const res = (await fn()) as { success: boolean; error?: string };
      if (res && 'success' in res && !res.success) {
        setError(res.error || `${label} failed`);
      } else {
        setNotice(`${label} completed`);
      }
      await loadAll();
    } catch (e: any) {
      setError(e.message || `${label} failed`);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="p-8 max-w-5xl mx-auto space-y-8 text-zinc-100">
      <div className="border-b border-zinc-800 pb-5 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Local Runtimes</span>
            <span className="text-xs px-2.5 py-1 rounded-full font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              Phase 16
            </span>
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            Detect and manage loopback inference runtimes (Ollama, llama.cpp server, local OpenAI-compatible endpoints)
          </p>
        </div>
        <div className="flex gap-2">
          <Link
            href="/settings/local-models"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
          >
            Local Models →
          </Link>
          <Link
            href="/settings/models"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
          >
            Model Registry
          </Link>
        </div>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">{error}</div>
      )}
      {notice && (
        <div className="p-4 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm">{notice}</div>
      )}

      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
        <h3 className="text-sm font-semibold text-zinc-200">Discovery & Registration</h3>
        <p className="text-xs text-zinc-400">
          Discovery probes loopback endpoints only and always passes through the network security boundary.
          Octrex never executes discovered binaries — start your runtime (e.g. <code className="font-mono">ollama serve</code>) and register its endpoint.
        </p>
        <div className="flex flex-wrap gap-2">
          <button
            disabled={busy}
            onClick={() => runAction('Discovery', () => backendClient.discoverLocalRuntimes({ include_defaults: true }))}
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-blue-500/10 hover:bg-blue-500/20 disabled:opacity-50 text-blue-400 border border-blue-500/20 transition"
          >
            Discover Default Loopback Runtimes
          </button>
          <button
            disabled={busy}
            onClick={loadAll}
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-200 border border-zinc-700 transition"
          >
            Refresh
          </button>
        </div>
        <div className="grid sm:grid-cols-4 gap-2">
          <input
            value={newEndpoint}
            onChange={(e) => setNewEndpoint(e.target.value)}
            placeholder="http://127.0.0.1:11434"
            className="sm:col-span-2 px-3 py-2 rounded-lg text-xs bg-zinc-950 border border-zinc-700 text-zinc-200 font-mono"
          />
          <select
            value={newType}
            onChange={(e) => setNewType(e.target.value)}
            className="px-3 py-2 rounded-lg text-xs bg-zinc-950 border border-zinc-700 text-zinc-200"
          >
            <option value="ollama">Ollama</option>
            <option value="llama_cpp_server">llama.cpp server</option>
            <option value="local_openai_compatible">Local OpenAI-compatible</option>
            <option value="other">Other</option>
          </select>
          <input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="Display name (optional)"
            className="px-3 py-2 rounded-lg text-xs bg-zinc-950 border border-zinc-700 text-zinc-200"
          />
        </div>
        <button
          disabled={busy || !newEndpoint}
          onClick={() =>
            runAction('Registration', () =>
              backendClient.registerLocalRuntime({ runtime_type: newType, endpoint: newEndpoint, name: newName || undefined }),
            )
          }
          className="px-4 py-2 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 disabled:opacity-50 text-emerald-400 border border-emerald-500/20 transition"
        >
          Register Loopback Endpoint
        </button>
      </div>

      <div className="space-y-4">
        <h3 className="text-sm font-semibold text-zinc-200">Registered Runtimes ({runtimes.length})</h3>
        {isLoading ? (
          <div className="p-6 text-center text-zinc-500 text-sm animate-pulse">Loading local runtimes...</div>
        ) : runtimes.length === 0 ? (
          <div className="p-6 text-center border border-dashed border-zinc-800 rounded-lg text-zinc-500 text-sm">
            No local runtimes registered. Run discovery or register a loopback endpoint above.
          </div>
        ) : (
          runtimes.map((rt) => (
            <LocalRuntimeCard
              key={rt.id}
              runtime={rt}
              modelCount={modelCounts[rt.id]}
              busy={busy}
              onRefresh={(id) => runAction('Refresh', () => backendClient.refreshLocalRuntime(id))}
              onTest={(id) => runAction('Connection test', () => backendClient.testLocalRuntime(id))}
              onStart={(id) => runAction('Start', () => backendClient.startLocalRuntime(id))}
              onStop={(id) => runAction('Stop', () => backendClient.stopLocalRuntime(id))}
              onDiscoverModels={(id) => runAction('Model discovery', () => backendClient.discoverLocalModels(id))}
            />
          ))
        )}
      </div>

      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
        <h3 className="text-sm font-semibold text-zinc-200">Storage & Live Activity</h3>
        <div className="text-xs text-zinc-400">
          Managed model directory: <code className="font-mono text-zinc-300">{modelDir || '—'}</code>
        </div>
        <div className="text-xs text-zinc-400">
          Active calls: <strong className="text-zinc-200">{activeCalls.length}</strong> · Execution slots held:{' '}
          <strong className="text-zinc-200">{slotOccupants.length === 0 ? 'none' : slotOccupants.join(', ')}</strong>
        </div>
        {metrics.length > 0 && (
          <div className="space-y-2">
            {metrics.slice(0, 8).map((m) => (
              <div key={m.call_id} className="p-3 rounded-lg border border-zinc-800 bg-zinc-950/60 text-xs flex flex-wrap gap-x-4 gap-y-1">
                <span className="font-mono text-zinc-500">{m.call_id.slice(0, 18)}…</span>
                <span className={m.success ? 'text-emerald-400' : 'text-rose-400'}>{m.success ? 'OK' : 'FAILED'}</span>
                <span className="text-zinc-400">
                  TTFT: <strong className="text-zinc-200">{m.time_to_first_token_ms ?? '—'} ms</strong>
                </span>
                <span className="text-zinc-400">
                  Total: <strong className="text-zinc-200">{m.total_latency_ms ?? '—'} ms</strong>
                </span>
                <span className="text-zinc-400">
                  tok/s: <strong className="text-zinc-200">{m.tokens_per_second !== null && m.tokens_per_second !== undefined ? m.tokens_per_second.toFixed(1) : '—'}</strong>
                </span>
                {m.error && <span className="text-rose-400/90 break-all">{m.error}</span>}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
