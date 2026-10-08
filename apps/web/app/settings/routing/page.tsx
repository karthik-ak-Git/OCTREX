'use client';

import React, { useEffect, useState } from 'react';
import { backendClient } from '@/lib/backend/client';
import {
  RoutingDecision,
  RoutingMode,
  RoutingStatusResponse,
} from '@/lib/backend/types';
import { RoutingModeSelector } from '@/components/RoutingModeSelector';
import { RoutingDecisionInspector } from '@/components/RoutingDecisionInspector';

export default function RoutingSettingsPage() {
  const [mode, setMode] = useState<RoutingMode>('auto');
  const [status, setStatus] = useState<RoutingStatusResponse | null>(null);
  const [previewDecision, setPreviewDecision] = useState<RoutingDecision | null>(null);
  const [isEvaluating, setIsEvaluating] = useState(false);
  const [isInspectorOpen, setIsInspectorOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [purpose, setPurpose] = useState('coding');

  useEffect(() => {
    fetchStatus();
  }, []);

  const fetchStatus = async () => {
    try {
      const res = await backendClient.getRouterStatus();
      setStatus(res);
    } catch (e: any) {
      setError(e.message || 'Failed to load router status');
    }
  };

  const handlePreview = async () => {
    setIsEvaluating(true);
    setError(null);
    try {
      const res = await backendClient.previewRouting({
        purpose,
        routing_mode: mode,
      });
      if (res.success) {
        setPreviewDecision(res.decision);
      } else {
        setError(res.error || 'Routing preview failed');
      }
    } catch (e: any) {
      setError(e.message || 'Failed to execute routing preview');
    } finally {
      setIsEvaluating(false);
    }
  };

  return (
    <div className="p-8 max-w-5xl mx-auto space-y-8 text-zinc-100">
      {/* Header */}
      <div className="border-b border-zinc-800 pb-5">
        <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
          <span>Model Router & Policy Routing</span>
          <span className="text-xs px-2.5 py-1 rounded-full font-mono bg-blue-500/10 text-blue-400 border border-blue-500/20">
            Phase 12
          </span>
        </h1>
        <p className="text-sm text-zinc-400 mt-1">
          Authorization-aware execution routing, privacy gate integration, and candidate model scoring
        </p>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">
          {error}
        </div>
      )}

      {/* System Status Summary */}
      <div className="grid grid-cols-1 sm:grid-cols-4 gap-4">
        <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/50 space-y-1">
          <span className="text-xs font-semibold text-zinc-400 uppercase">Registered Models</span>
          <div className="text-2xl font-bold text-zinc-100">
            {status?.registered_models ?? '-'}
          </div>
        </div>
        <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/50 space-y-1">
          <span className="text-xs font-semibold text-zinc-400 uppercase">Providers</span>
          <div className="text-2xl font-bold text-zinc-100">
            {status?.registered_providers ?? '-'}
          </div>
        </div>
        <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/50 space-y-1">
          <span className="text-xs font-semibold text-zinc-400 uppercase">Hardware Bounds</span>
          <div className="text-base font-semibold text-emerald-400 capitalize">
            {status?.hardware_confidence ?? 'Exact'}
          </div>
        </div>
        <div className="p-4 rounded-lg border border-zinc-800 bg-zinc-900/50 space-y-1">
          <span className="text-xs font-semibold text-zinc-400 uppercase">Cloud Fallback</span>
          <div className="text-base font-semibold text-rose-400">Disabled</div>
        </div>
      </div>

      {/* Routing Mode Selector */}
      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
        <RoutingModeSelector value={mode} onChange={setMode} />
      </div>

      {/* Interactive Preview Diagnostics */}
      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
        <h3 className="text-sm font-semibold text-zinc-200">
          Diagnostic Routing Preview (Non-destructive)
        </h3>
        <p className="text-xs text-zinc-400">
          Simulate model selection and authorization pipeline without executing LLM calls or sending payloads online.
        </p>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 pt-2">
          <select
            value={purpose}
            onChange={(e) => setPurpose(e.target.value)}
            className="bg-zinc-950 border border-zinc-800 text-zinc-200 text-xs rounded-lg px-3 py-2.5"
          >
            <option value="coding">Coding & Software Architecture</option>
            <option value="general">General Chat & Reasoning</option>
            <option value="document_extraction">Document & Vision Extraction</option>
            <option value="tool_calling">Autonomous Agent & Tool Execution</option>
          </select>

          <button
            onClick={handlePreview}
            disabled={isEvaluating}
            className="px-4 py-2.5 rounded-lg text-xs font-medium bg-blue-600 hover:bg-blue-500 text-white transition disabled:opacity-50"
          >
            {isEvaluating ? 'Simulating Pipeline...' : 'Run Diagnostic Preview'}
          </button>
        </div>

        {previewDecision && (
          <div className="mt-4 p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-zinc-300">
                Preview Result: <strong className="text-blue-400">{previewDecision.state}</strong>
              </span>
              <button
                onClick={() => setIsInspectorOpen(true)}
                className="text-xs text-blue-400 hover:underline font-medium"
              >
                Inspect Routing Evidence →
              </button>
            </div>
            <p className="text-xs font-mono text-zinc-400 bg-zinc-900/80 p-2.5 rounded border border-zinc-800/80">
              {previewDecision.reason}
            </p>
          </div>
        )}
      </div>

      <RoutingDecisionInspector
        decision={previewDecision}
        isOpen={isInspectorOpen}
        onClose={() => setIsInspectorOpen(false)}
      />
    </div>
  );
}
