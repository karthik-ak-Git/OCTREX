'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { backendClient } from '@/lib/backend/client';
import { HardwareProfile, ModelDescriptor } from '@/lib/backend/types';
import { HardwareCompatibilityCard } from '@/components/HardwareCompatibilityCard';
import { ModelCompatibilityBadge } from '@/components/ModelCompatibilityBadge';

export default function ModelCompatibilityPage() {
  const [profile, setProfile] = useState<HardwareProfile | null>(null);
  const [recommendedModels, setRecommendedModels] = useState<ModelDescriptor[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [routerStatus, recommendedRes] = await Promise.all([
        backendClient.getRouterStatus().catch(() => null),
        backendClient.getRecommendedModels().catch(() => null),
      ]);
      if (recommendedRes && 'models' in recommendedRes) {
        setRecommendedModels(recommendedRes.models || []);
      }
    } catch (e: any) {
      setError(e.message || 'Failed to load hardware compatibility details');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-8 max-w-5xl mx-auto space-y-8 text-zinc-100">
      {/* Header */}
      <div className="border-b border-zinc-800 pb-5 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-3">
            <span>Hardware Compatibility & Capacity</span>
            <span className="text-xs px-2.5 py-1 rounded-full font-mono bg-blue-500/10 text-blue-400 border border-blue-500/20">
              Phase 5 / Phase 12
            </span>
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            Local hardware constraints, VRAM bounds, CPU memory limits, and recommended local/on-prem candidate models
          </p>
        </div>
        <Link
          href="/settings/models"
          className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
        >
          ← Back to Model Registry
        </Link>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">
          {error}
        </div>
      )}

      {/* Hardware Profile Snapshot */}
      <HardwareCompatibilityCard profile={profile} />

      {/* Recommended Models Section */}
      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
        <h3 className="text-sm font-semibold text-zinc-200">
          Hardware-Compatible Recommended Models
        </h3>
        <p className="text-xs text-zinc-400">
          Models evaluated against your active hardware profile (VRAM, CPU RAM, accelerator availability) that meet local execution thresholds.
        </p>

        {isLoading ? (
          <div className="p-6 text-center text-zinc-500 text-sm animate-pulse">
            Analyzing local hardware capability and filtering model candidates...
          </div>
        ) : recommendedModels.length === 0 ? (
          <div className="p-6 text-center border border-dashed border-zinc-800 rounded-lg text-zinc-500 text-sm">
            No specific local model recommendations found for current hardware bounds.
          </div>
        ) : (
          <div className="space-y-3">
            {recommendedModels.map((model) => (
              <div
                key={model.id}
                className="p-4 rounded-lg border border-zinc-800 bg-zinc-950/60 flex items-center justify-between"
              >
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-sm text-zinc-100">{model.display_name}</span>
                    <span className="text-xs font-mono text-zinc-500">({model.id})</span>
                    <ModelCompatibilityBadge executionMode={model.execution_mode} />
                  </div>
                  <div className="text-xs text-zinc-400 flex items-center gap-4">
                    <span>Provider: <strong className="text-zinc-300">{model.provider_id}</strong></span>
                    <span>Context Window: <strong className="text-zinc-300">{model.context_window?.toLocaleString() || 'N/A'} tokens</strong></span>
                  </div>
                </div>
                <div className="text-xs px-2.5 py-1 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                  Compatible
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
