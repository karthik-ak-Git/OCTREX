'use client';

import React from 'react';
import { MemoryCandidate } from '@/lib/backend/types';
import { MemoryScopeBadge, MemoryClassificationBadge } from './SkillMemoryBadges';

export function MemoryCandidateReview({
  candidate,
  onApprove,
  onReject,
  busy,
}: {
  candidate: MemoryCandidate;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  busy: boolean;
}) {
  return (
    <div className="p-4 bg-amber-950/20 border border-amber-800/40 rounded-lg space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-amber-200">Pending approval — {candidate.id}</span>
        <span className="flex items-center space-x-1.5">
          <MemoryScopeBadge scope={candidate.scope} />
          <MemoryClassificationBadge classification={candidate.classification} />
        </span>
      </div>
      <p className="text-xs text-zinc-300">{candidate.content}</p>
      <div className="text-[11px] text-zinc-500 font-mono">
        {candidate.mem_type} · {candidate.source} · conf {candidate.confidence.toFixed(2)}
      </div>
      <div className="flex items-center space-x-2 pt-1">
        <button
          disabled={busy}
          onClick={() => onApprove(candidate.id)}
          className="px-3 py-1 bg-emerald-950/60 hover:bg-emerald-900 border border-emerald-800/50 text-emerald-300 rounded text-xs"
        >
          Approve
        </button>
        <button
          disabled={busy}
          onClick={() => onReject(candidate.id)}
          className="px-3 py-1 bg-rose-950/60 hover:bg-rose-900 border border-rose-800/50 text-rose-300 rounded text-xs"
        >
          Reject
        </button>
      </div>
    </div>
  );
}

export function MemoryInspector({ item }: { item: Record<string, unknown> | null }) {
  if (!item) return <div className="text-xs text-zinc-500">Select a memory item.</div>;
  return (
    <div className="p-4 bg-zinc-900 border border-zinc-800 rounded-lg text-xs space-y-2">
      <div className="font-mono text-zinc-500">{String(item['id'] ?? '')}</div>
      <div className="text-zinc-200">{String(item['content_preview'] ?? item['content'] ?? '')}</div>
      <pre className="p-2 bg-zinc-950 border border-zinc-800 rounded text-[11px] text-zinc-400 overflow-auto">
        {JSON.stringify(item, null, 2)}
      </pre>
      <p className="text-zinc-600">Raw secrets are never rendered. SECRET content is redacted.</p>
    </div>
  );
}
