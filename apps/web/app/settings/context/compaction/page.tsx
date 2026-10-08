'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, Layers, RefreshCw, CheckCircle2, AlertTriangle, ShieldAlert } from 'lucide-react';
import { backendClient } from '../../../../lib/backend/client';
import { CompactionStatus } from '../../../../lib/backend/types';

export default function CompactionPage() {
  const [sessionId, setSessionId] = useState<string>('global');
  const [modelId, setModelId] = useState<string>('mock-model');
  const [compaction, setCompaction] = useState<CompactionStatus | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  const triggerCompaction = async () => {
    setLoading(true);
    setError('');
    try {
      const res = await backendClient.compactContext({
        session_id: sessionId,
        model_id: modelId,
      });
      if (res.success && res.compaction) {
        setCompaction(res.compaction);
      } else {
        setError(res.error || 'Compaction failed');
      }
    } catch (err: any) {
      setError(err.message || 'Failed to trigger compaction');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#eaedf3] text-slate-900 p-8 flex justify-center">
      <div className="w-[680px] space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between bg-white/80 backdrop-blur-xl border border-white/90 p-5 rounded-3xl shadow-sm">
          <div className="flex items-center space-x-3">
            <Link href="/settings/context" className="p-2 text-slate-400 hover:text-slate-800 rounded-xl hover:bg-slate-100 transition-all">
              <ArrowLeft className="w-5 h-5" />
            </Link>
            <div>
              <h1 className="text-xl font-extrabold text-slate-900 tracking-tight">Compaction Engine Status</h1>
              <p className="text-xs text-slate-500 font-medium">Bounded structured context reduction & summary generation</p>
            </div>
          </div>
        </div>

        {/* Trigger Compaction Form */}
        <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl shadow-sm space-y-4">
          <h2 className="text-sm font-extrabold text-slate-900">Trigger Bounded Compaction</h2>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-[10px] font-extrabold uppercase text-slate-400 mb-1">Session ID</label>
              <input
                type="text"
                value={sessionId}
                onChange={(e) => setSessionId(e.target.value)}
                className="w-full px-4 py-2 bg-white border border-slate-200 rounded-2xl text-xs font-mono text-slate-800 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
              />
            </div>
            <div>
              <label className="block text-[10px] font-extrabold uppercase text-slate-400 mb-1">Target Model ID</label>
              <input
                type="text"
                value={modelId}
                onChange={(e) => setModelId(e.target.value)}
                className="w-full px-4 py-2 bg-white border border-slate-200 rounded-2xl text-xs font-mono text-slate-800 focus:outline-none focus:ring-2 focus:ring-slate-900/10"
              />
            </div>
          </div>

          <button
            onClick={triggerCompaction}
            disabled={loading}
            className="w-full py-3 bg-purple-900 hover:bg-purple-800 text-white font-extrabold text-xs rounded-2xl shadow-sm transition-all flex items-center justify-center space-x-2 disabled:opacity-50"
          >
            <Layers className="w-4 h-4" />
            <span>{loading ? 'Compacting Context...' : 'Run Compaction Engine'}</span>
          </button>
        </div>

        {error && (
          <div className="p-4 bg-rose-50 border border-rose-200 text-rose-800 rounded-2xl text-xs font-medium flex items-center space-x-2">
            <AlertTriangle className="w-4 h-4 text-rose-600" />
            <span>{error}</span>
          </div>
        )}

        {/* Compaction Result Card */}
        {compaction && (
          <div className="bg-white/80 backdrop-blur-xl border border-white/90 p-6 rounded-3xl space-y-5 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2.5">
                <div className="w-9 h-9 rounded-2xl bg-purple-900 text-white flex items-center justify-center font-bold shadow-xs">
                  <CheckCircle2 className="w-5 h-5" />
                </div>
                <div>
                  <h2 className="text-sm font-extrabold text-slate-900">Compaction Result</h2>
                  <p className="text-xs text-slate-500 font-medium">Completed in {compaction.rounds} round(s)</p>
                </div>
              </div>
              <span className="px-3 py-1 rounded-full text-xs font-extrabold bg-purple-100 text-purple-800 border border-purple-300 font-mono">
                {compaction.tokens_saved.toLocaleString()} tokens saved
              </span>
            </div>

            <div className="grid grid-cols-3 gap-3 font-mono text-xs">
              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Tokens Before</div>
                <div className="text-lg font-extrabold text-slate-900">{compaction.tokens_before.toLocaleString()}</div>
              </div>
              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Tokens After</div>
                <div className="text-lg font-extrabold text-slate-900">{compaction.tokens_after.toLocaleString()}</div>
              </div>
              <div className="p-4 bg-slate-50 border border-slate-200/80 rounded-2xl">
                <div className="text-[10px] font-extrabold uppercase text-slate-400 font-sans">Items Compacted</div>
                <div className="text-lg font-extrabold text-slate-900">{compaction.items_compacted}</div>
              </div>
            </div>

            <div className="p-4 bg-purple-50 border border-purple-200 rounded-2xl text-xs space-y-1">
              <div className="font-extrabold text-purple-900 flex items-center space-x-1.5">
                <ShieldAlert className="w-4 h-4 text-purple-700" />
                <span>Classification Maintained: {compaction.classification}</span>
              </div>
              <p className="text-purple-700 font-medium">
                Compaction summaries inherit max(source item classifications) to ensure zero security downgrades.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
