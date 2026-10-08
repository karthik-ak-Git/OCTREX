'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { backendClient } from '@/lib/backend/client';
import { LocalCompatibilityReport, LocalModelRecord } from '@/lib/backend/types';
import { LocalModelCompatibilityPanel, LocalModelStateBadge } from '@/components/LocalModelCards';
import { LocalRuntimeHealthBadge } from '@/components/LocalRuntimeCard';

export default function LocalModelDetailClient({ modelId }: { modelId: string }) {
  const id = modelId;
  const [model, setModel] = useState<LocalModelRecord | null>(null);
  const [compat, setCompat] = useState<LocalCompatibilityReport | null>(null);
  const [inputTokens, setInputTokens] = useState('2048');
  const [outputTokens, setOutputTokens] = useState('1024');
  const [isLoading, setIsLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [prompt, setPrompt] = useState('');
  const [answer, setAnswer] = useState<string | null>(null);

  useEffect(() => {
    loadModel();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  const loadModel = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const res = await backendClient.getLocalModel(id);
      if (!res.success || !res.model) throw new Error(res.error || 'Model not found');
      setModel(res.model);
      await loadCompat(res.model.id);
    } catch (e: any) {
      setError(e.message || 'Failed to load model');
    } finally {
      setIsLoading(false);
    }
  };

  const loadCompat = async (targetId?: string) => {
    setBusy(true);
    try {
      const res = await backendClient.getLocalModelCompatibility(
        targetId || id,
        Number(inputTokens) || 1024,
        outputTokens ? Number(outputTokens) : undefined,
      );
      if (!res.success) throw new Error(res.error || 'Compatibility check failed');
      setCompat(res.compatibility || null);
    } catch (e: any) {
      setError(e.message || 'Compatibility check failed');
    } finally {
      setBusy(false);
    }
  };

  const runInference = async () => {
    if (!model || !prompt.trim()) return;
    setBusy(true);
    setError(null);
    setAnswer(null);
    try {
      const preview = await backendClient.previewLocalInference({
        registry_model_id: model.registry_model_id,
        input_tokens: Number(inputTokens) || 1024,
        output_tokens: outputTokens ? Number(outputTokens) : undefined,
      });
      const pv = preview.preview as { routable?: boolean } | undefined;
      if (!preview.success || !pv?.routable) {
        throw new Error('Model is not currently routable — see compatibility verdict above.');
      }
      const res = await backendClient.executeLocalInference({
        registry_model_id: model.registry_model_id,
        messages: [{ role: 'user', content: prompt }],
        max_output_tokens: outputTokens ? Number(outputTokens) : undefined,
      });
      if (!res.success) {
        throw new Error(`${res.error || 'Inference failed'}${res.no_cloud_fallback ? ' (no cloud fallback was attempted)' : ''}`);
      }
      setAnswer(res.response?.content || '(empty response)');
    } catch (e: any) {
      setError(e.message || 'Inference failed');
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="p-8 max-w-5xl mx-auto space-y-8 text-zinc-100">
      <div className="border-b border-zinc-800 pb-5 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white">Local Model Detail</h1>
          <p className="text-sm text-zinc-400 mt-1 font-mono">{id}</p>
        </div>
        <Link
          href="/settings/local-models"
          className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
        >
          ← Local Models
        </Link>
      </div>

      {error && <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">{error}</div>}

      {isLoading ? (
        <div className="p-6 text-center text-zinc-500 text-sm animate-pulse">Loading model...</div>
      ) : model ? (
        <>
          <div className="p-5 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-2">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="font-semibold text-zinc-100">{model.display_name}</span>
              <LocalModelStateBadge state={model.state} />
              <LocalRuntimeHealthBadge health={model.health} />
            </div>
            <div className="text-xs text-zinc-400 grid sm:grid-cols-2 gap-x-6 gap-y-1">
              <span>Registry ID: <code className="font-mono text-zinc-300">{model.registry_model_id}</code></span>
              <span>Runtime: <code className="font-mono text-zinc-300">{model.runtime_id}</code></span>
              <span>Context window: <strong className="text-zinc-200">{model.context_window?.toLocaleString() ?? 'Unknown'}</strong></span>
              <span>Quantization: <strong className="text-zinc-200">{model.quantization ?? 'Unknown'}</strong></span>
              <span>Parameters: <strong className="text-zinc-200">{model.parameter_count_billions ?? 'Unknown'}{model.parameter_count_billions ? 'B' : ''}</strong></span>
              <span>Format: <strong className="text-zinc-200">{model.model_format ?? 'Unknown'}</strong></span>
            </div>
            {model.metadata?.requirement_warning && (
              <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">
                Requirement sanity warning: {model.metadata.requirement_warning}
              </div>
            )}
          </div>

          <div className="p-5 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-3">
            <h3 className="text-sm font-semibold text-zinc-200">Compatibility Budget</h3>
            <div className="flex flex-wrap gap-2 items-end">
              <label className="text-xs text-zinc-400">
                Input tokens
                <input value={inputTokens} onChange={(e) => setInputTokens(e.target.value)} className="ml-2 px-3 py-1.5 rounded-lg bg-zinc-950 border border-zinc-700 text-zinc-200 font-mono w-32" />
              </label>
              <label className="text-xs text-zinc-400">
                Reserved output
                <input value={outputTokens} onChange={(e) => setOutputTokens(e.target.value)} className="ml-2 px-3 py-1.5 rounded-lg bg-zinc-950 border border-zinc-700 text-zinc-200 font-mono w-32" />
              </label>
              <button
                disabled={busy}
                onClick={() => loadCompat()}
                className="px-4 py-1.5 rounded-lg text-xs font-semibold bg-blue-500/10 hover:bg-blue-500/20 disabled:opacity-50 text-blue-400 border border-blue-500/20 transition"
              >
                Re-evaluate
              </button>
            </div>
            {compat && <LocalModelCompatibilityPanel report={compat} />}
          </div>

          <div className="p-5 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-3">
            <h3 className="text-sm font-semibold text-zinc-200">Try Local Inference</h3>
            <p className="text-xs text-zinc-500">
              Executes only against this local model via ModelRuntime. Local failure never routes to cloud.
            </p>
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Type a prompt to run locally…"
              rows={3}
              className="w-full px-3 py-2 rounded-lg text-sm bg-zinc-950 border border-zinc-700 text-zinc-200"
            />
            <button
              disabled={busy || !prompt.trim()}
              onClick={runInference}
              className="px-4 py-2 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 disabled:opacity-50 text-emerald-400 border border-emerald-500/20 transition"
            >
              {busy ? 'Running…' : 'Run Locally'}
            </button>
            {answer !== null && (
              <div className="p-4 rounded-lg border border-zinc-700 bg-zinc-950 text-sm text-zinc-200 whitespace-pre-wrap max-h-96 overflow-y-auto">
                {answer}
              </div>
            )}
          </div>
        </>
      ) : null}
    </div>
  );
}
