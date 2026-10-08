'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { backendClient } from '@/lib/backend/client';
import { ModelDescriptor } from '@/lib/backend/types';
import { ModelCompatibilityBadge } from '@/components/ModelCompatibilityBadge';
import { ModelCandidateList } from '@/components/ModelCandidateList';

export default function ModelsSettingsPage() {
  const [models, setModels] = useState<ModelDescriptor[]>([]);
  const [selectedModelId, setSelectedModelId] = useState<string | undefined>(undefined);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchModels();
  }, []);

  const fetchModels = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const res = await backendClient.getCompatibleModels();
      setModels(res.models || []);
    } catch (e: any) {
      setError(e.message || 'Failed to load registered models');
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
            <span>Model Registry & Candidates</span>
            <span className="text-xs px-2.5 py-1 rounded-full font-mono bg-blue-500/10 text-blue-400 border border-blue-500/20">
              Phase 12 Router
            </span>
          </h1>
          <p className="text-sm text-zinc-400 mt-1">
            Browse registered models across Local, On-Premise, and Online execution providers
          </p>
        </div>
        <div className="flex gap-2">
          <Link
            href="/settings/local-runtime"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-400 border border-emerald-500/20 transition"
          >
            Local Runtimes →
          </Link>
          <Link
            href="/settings/local-models"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-400 border border-emerald-500/20 transition"
          >
            Local Models →
          </Link>
          <Link
            href="/settings/models/compatibility"
            className="px-4 py-2 rounded-lg text-xs font-semibold bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition"
          >
            View Hardware Compatibility →
          </Link>
        </div>
      </div>

      {error && (
        <div className="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-sm">
          {error}
        </div>
      )}

      {/* Model Candidate List */}
      <div className="p-6 rounded-xl border border-zinc-800 bg-zinc-900/40 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-zinc-200">
            Registered Models ({models.length})
          </h3>
          <button
            onClick={fetchModels}
            className="text-xs text-blue-400 hover:underline font-medium"
          >
            Refresh Candidates
          </button>
        </div>

        <ModelCandidateList
          models={models}
          selectedModelId={selectedModelId}
          onSelectModel={(id) => setSelectedModelId(id)}
          isLoading={isLoading}
        />
      </div>
    </div>
  );
}
