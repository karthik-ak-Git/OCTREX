'use client';

import React, { useEffect, useState } from 'react';
import { MemoryCandidate, MemoryResultRow } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';
import { MemoryScopeBadge, MemoryClassificationBadge } from '@/components/SkillMemoryBadges';
import { MemoryCandidateReview, MemoryInspector } from '@/components/MemoryReview';

export default function MemoryPage() {
  const [results, setResults] = useState<MemoryResultRow[]>([]);
  const [candidates, setCandidates] = useState<MemoryCandidate[]>([]);
  const [selected, setSelected] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [queryText, setQueryText] = useState('');

  const fetchAll = async () => {
    try {
      setLoading(true);
      const [mem, cands] = await Promise.all([
        backendClient.queryMemory({ classification_ceiling: 'INTERNAL', limit: 20, query_text: queryText || undefined }),
        backendClient.listMemoryCandidates('PENDING'),
      ]);
      if (mem.success) setResults(mem.results);
      if (cands.success) setCandidates(cands.candidates);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Failed loading memory');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAll();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const approve = async (id: string) => {
    try {
      setBusy(true);
      await backendClient.approveMemoryCandidate(id);
      await fetchAll();
    } finally {
      setBusy(false);
    }
  };

  const reject = async (id: string) => {
    try {
      setBusy(true);
      await backendClient.rejectMemoryCandidate(id);
      await fetchAll();
    } finally {
      setBusy(false);
    }
  };

  const remove = async (id: string) => {
    try {
      setBusy(true);
      await backendClient.deleteMemory(id);
      await fetchAll();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-8 space-y-6">
      <div className="border-b border-zinc-800 pb-5">
        <h1 className="text-2xl font-bold text-white">Agent Memory</h1>
        <p className="text-sm text-zinc-400 mt-1">
          Scoped, classified, provenance-tracked facts — never transcripts. Model-generated memory stays untrusted until approved. Memory never outranks policy.
        </p>
      </div>
      <div className="flex items-center space-x-2">
        <input
          value={queryText}
          onChange={(e) => setQueryText(e.target.value)}
          placeholder="Filter memory…"
          className="px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded text-xs w-64"
        />
        <button onClick={fetchAll} className="px-3 py-1.5 bg-zinc-800 rounded text-xs">Search</button>
      </div>
      {loading ? (
        <div className="text-sm text-zinc-500">Loading memory…</div>
      ) : error ? (
        <div className="p-4 bg-red-950/40 border border-red-800/50 rounded text-red-300 text-sm">{error}</div>
      ) : (
        <div className="space-y-6">
          <div>
            <h3 className="text-sm font-semibold mb-2">Pending approval ({candidates.length})</h3>
            <div className="space-y-2">
              {candidates.length === 0 && <div className="text-xs text-zinc-600">No pending candidates.</div>}
              {candidates.map((c) => (
                <MemoryCandidateReview key={c.id} candidate={c} onApprove={approve} onReject={reject} busy={busy} />
              ))}
            </div>
          </div>
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            <div className="lg:col-span-7 bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
              <div className="px-5 py-3 border-b border-zinc-800">Memory ({results.length})</div>
              <div className="divide-y divide-zinc-800/60 max-h-[500px] overflow-y-auto">
                {results.map((r) => (
                  <div key={r.id} onClick={() => setSelected(r as unknown as Record<string, unknown>)} className="p-3 hover:bg-zinc-800/30 cursor-pointer">
                    <div className="flex items-center justify-between">
                      <span className="font-mono text-xs text-zinc-300">{r.id}</span>
                      <span className="flex items-center space-x-1.5">
                        <MemoryScopeBadge scope={r.scope} />
                        <MemoryClassificationBadge classification={r.classification} />
                      </span>
                    </div>
                    <p className="text-xs text-zinc-400 mt-1">{r.content_preview}</p>
                    <div className="text-[11px] text-zinc-600 font-mono mt-1">{r.type} · {r.source} · rel {r.relevance.toFixed(1)}</div>
                    <button onClick={(e) => { e.stopPropagation(); remove(r.id); }} className="mt-1 px-2 py-0.5 bg-rose-950/60 border border-rose-800/50 text-rose-300 rounded text-[11px]">Delete</button>
                  </div>
                ))}
              </div>
            </div>
            <div className="lg:col-span-5">
              <MemoryInspector item={selected} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
