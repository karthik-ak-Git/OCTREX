'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { backendClient } from '@/lib/backend/client';
import { LocalModelRecord, LocalRoutingCandidateExplanation, LocalRuntimeDescriptor } from '@/lib/backend/types';
import { LocalModelCard } from '@/components/LocalModelCards';

export default function LocalModelsPage() {
  const [models, setModels] = useState<LocalModelRecord[]>([]);
  const [runtimes, setRuntimes] = useState<LocalRuntimeDescriptor[]>([]);
  const [explanations, setExplanations] = useState<LocalRoutingCandidateExplanation[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [showRegister, setShowRegister] = useState(false);
  const [showDownload, setShowDownload] = useState(false);

  const [reg, setReg] = useState({
    runtime_id: '',
    model_identifier: '',
    display_name: '',
    context_window: '8192',
    quantization: 'Q4_K_M',
    parameter_count_billions: '7',
    architecture: '',
    model_format: 'gguf',
    license: '',
    source: '',
  });

  const [dl, setDl] = useState({
    source: 'configured-endpoint',
    url: '',
    file_name: '',
    checksum_sha256: '',
    consent: false,
  });

  useEffect(() => {
    loadAll();
  }, []);

  const loadAll = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [m, r, e] = await Promise.all([
        backendClient.listLocalModels(),
        backendClient.listLocalRuntimes().catch(() => ({ runtimes: [] as LocalRuntimeDescriptor[] })),
        backendClient.explainLocalRouting(1024, 512).catch(() => null),
      ]);
      setModels(m.models || []);
      setRuntimes(r.runtimes || []);
      if (e) setExplanations(e.candidates || []);
      if (!reg.runtime_id && r.runtimes && r.runtimes.length > 0) {
        setReg((prev) => ({ ...prev, runtime_id: r.runtimes[0].id }));
      }
    } catch (e: any) {
      setError(e.message || 'Failed to load local models');
    } finally {
      setIsLoading(false);
    }
  };

  const runAction = async (label: string, fn: () => Promise<unknown>) => {
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

  const submitRegister = async () => {
    await runAction('Model registration', () =>
      backendClient.registerLocalModel({
        runtime_id: reg.runtime_id,
        model_identifier: reg.model_identifier,
        display_name: reg.display_name || undefined,
        context_window: reg.context_window ? Number(reg.context_window) : undefined,
        quantization: reg.quantization || undefined,
        parameter_count_billions: reg.parameter_count_billions ? Number(reg.parameter_count_billions) : undefined,
        architecture: reg.architecture || undefined,
        model_format: reg.model_format || undefined,
        license: reg.license || undefined,
        source: reg.source || undefined,
      }),
    );
    setShowRegister(false);
  };

  const submitDownload = async () => {
    await runAction('Model download', () =>
      backendClient.downloadLocalModel({
        source: dl.source,
        url: dl.url,
        file_name: dl.file_name,
        checksum_sha256: dl.checksum_sha256 || undefined,
        consent: dl.consent,
      }),
    );
    setShowDownload(false);
  };

  return (
    <div className="p-8 max-w-5xl mx-auto space-y-8 text-zinc-100">
      <div className="border-b border-zinc-800 pb-5 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Local Models</span>
            <span className="text-xs px-2.5 py-1 rounded-full font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              Phase 16
            </span>
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            Lifecycle-managed local models synced into the existing ModelRegistry — the ModelRouter stays authoritative
          </p>
        </div>
        <div className="flex gap-2">
          <Link
            href="/settings/local-runtime"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
          >
            ← Local Runtimes
          </Link>
          <Link
            href="/settings/models/compatibility"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
          >
            Hardware Compatibility
          </Link>
        </div>
      </div>

      {error && <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">{error}</div>}
      {notice && <div className="p-4 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm">{notice}</div>}

      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-3">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-sm font-semibold text-zinc-200">Managed Models ({models.length})</h3>
          <div className="flex gap-2">
            <button
              onClick={() => setShowRegister((v) => !v)}
              className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-blue-500/10 hover:bg-blue-500/20 text-blue-400 border border-blue-500/20 transition"
            >
              Register Model
            </button>
            <button
              onClick={() => setShowDownload((v) => !v)}
              className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/20 transition"
            >
              Download Model
            </button>
            <button onClick={loadAll} className="text-xs text-blue-400 hover:underline font-medium">
              Refresh
            </button>
          </div>
        </div>

        {showRegister && (
          <div className="p-4 rounded-lg border border-zinc-700 bg-zinc-950/60 grid sm:grid-cols-2 gap-2 text-xs">
            <select value={reg.runtime_id} onChange={(e) => setReg({ ...reg, runtime_id: e.target.value })} className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 sm:col-span-2">
              {runtimes.map((r) => (
                <option key={r.id} value={r.id}>{r.name} ({r.endpoint})</option>
              ))}
            </select>
            <input value={reg.model_identifier} onChange={(e) => setReg({ ...reg, model_identifier: e.target.value })} placeholder="Model identifier (e.g. llama3.1:8b)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            <input value={reg.display_name} onChange={(e) => setReg({ ...reg, display_name: e.target.value })} placeholder="Display name (optional)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200" />
            <input value={reg.context_window} onChange={(e) => setReg({ ...reg, context_window: e.target.value })} placeholder="Context window" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            <input value={reg.quantization} onChange={(e) => setReg({ ...reg, quantization: e.target.value })} placeholder="Quantization (e.g. Q4_K_M)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            <input value={reg.parameter_count_billions} onChange={(e) => setReg({ ...reg, parameter_count_billions: e.target.value })} placeholder="Params (billions)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            <input value={reg.model_format} onChange={(e) => setReg({ ...reg, model_format: e.target.value })} placeholder="Format (gguf/safetensors/ollama/...)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            <div className="sm:col-span-2 flex gap-2">
              <button disabled={busy || !reg.runtime_id || !reg.model_identifier} onClick={submitRegister} className="px-4 py-2 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 disabled:opacity-50 text-emerald-400 border border-emerald-500/20 transition">
                Confirm Registration
              </button>
              <button onClick={() => setShowRegister(false)} className="px-4 py-2 rounded-lg text-xs text-zinc-400 hover:underline">Cancel</button>
            </div>
          </div>
        )}

        {showDownload && (
          <div className="p-4 rounded-lg border border-amber-500/20 bg-amber-500/5 space-y-2 text-xs">
            <p className="text-amber-300/90">
              Downloads are explicit, consent-gated, policy-checked, size-capped, and land only in the managed model
              directory. Redirects are re-validated. No silent background downloads exist.
            </p>
            <input value={dl.url} onChange={(e) => setDl({ ...dl, url: e.target.value })} placeholder="https://…/model.gguf" className="w-full px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            <div className="grid sm:grid-cols-2 gap-2">
              <input value={dl.file_name} onChange={(e) => setDl({ ...dl, file_name: e.target.value })} placeholder="file name (e.g. model.gguf)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
              <input value={dl.checksum_sha256} onChange={(e) => setDl({ ...dl, checksum_sha256: e.target.value })} placeholder="SHA-256 (optional, 64 hex chars)" className="px-3 py-2 rounded-lg bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono" />
            </div>
            <label className="flex items-center gap-2 text-zinc-300">
              <input type="checkbox" checked={dl.consent} onChange={(e) => setDl({ ...dl, consent: e.target.checked })} />
              I explicitly consent to this download (required)
            </label>
            <div className="flex gap-2">
              <button disabled={busy || !dl.consent || !dl.url || !dl.file_name} onClick={submitDownload} className="px-4 py-2 rounded-lg text-xs font-semibold bg-amber-500/10 hover:bg-amber-500/20 disabled:opacity-50 text-amber-400 border border-amber-500/20 transition">
                Start Explicit Download
              </button>
              <button onClick={() => setShowDownload(false)} className="px-4 py-2 rounded-lg text-xs text-zinc-400 hover:underline">Cancel</button>
            </div>
          </div>
        )}

        {isLoading ? (
          <div className="p-6 text-center text-zinc-500 text-sm animate-pulse">Loading local models...</div>
        ) : models.length === 0 ? (
          <div className="p-6 text-center border border-dashed border-zinc-800 rounded-lg text-zinc-500 text-sm">
            No local models yet. Discover them from a runtime or register one explicitly.
          </div>
        ) : (
          <div className="space-y-3">
            {models.map((m) => (
              <LocalModelCard
                key={m.id}
                model={m}
                busy={busy}
                onEnable={(id) => runAction('Enable', () => backendClient.enableLocalModel(id))}
                onDisable={(id) => runAction('Disable', () => backendClient.disableLocalModel(id))}
              />
            ))}
          </div>
        )}
      </div>

      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-3">
        <h3 className="text-sm font-semibold text-zinc-200">Why was this model (not) selected?</h3>
        <p className="text-xs text-zinc-400">
          Advisory diagnostics only — the ModelRouter remains the selection authority. Unknown health or requirements never auto-pass.
        </p>
        {explanations.length === 0 ? (
          <div className="text-xs text-zinc-500">No local candidates to explain yet.</div>
        ) : (
          <div className="space-y-2">
            {explanations.map((c) => (
              <div key={c.model_id} className="p-3 rounded-lg border border-zinc-800 bg-zinc-950/60 text-xs space-y-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="font-mono text-zinc-300">{c.registry_model_id}</span>
                  <span className={`px-2 py-0.5 rounded font-mono border ${c.routable ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border-rose-500/20'}`}>
                    {c.routable ? 'ROUTABLE' : 'REJECTED'}
                  </span>
                  <span className="text-zinc-500">state={c.state} health={c.health} hw={c.hardware_status}</span>
                </div>
                <ul className="text-zinc-500 list-disc list-inside">
                  {c.reasons.map((r, i) => (
                    <li key={i} className="break-words">{r}</li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
