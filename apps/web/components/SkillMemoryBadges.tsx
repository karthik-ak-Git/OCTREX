'use client';

import React from 'react';
import { SkillSource, SkillStatus, PrivacyClassification } from '@/lib/backend/types';

export function SkillSecurityBadge({ source, status }: { source: SkillSource; status: SkillStatus }) {
  const sourceColor =
    source === 'SYSTEM' || source === 'COMPANY'
      ? 'bg-emerald-950/60 border-emerald-800/50 text-emerald-300'
      : source === 'BUILTIN' || source === 'USER'
        ? 'bg-sky-950/60 border-sky-800/50 text-sky-300'
        : 'bg-amber-950/60 border-amber-800/50 text-amber-300';
  const statusColor =
    status === 'ACTIVE'
      ? 'bg-emerald-950/60 border-emerald-800/50 text-emerald-300'
      : status === 'DISABLED' || status === 'BLOCKED'
        ? 'bg-rose-950/60 border-rose-800/50 text-rose-300'
        : 'bg-zinc-800 border-zinc-700 text-zinc-300';
  return (
    <span className="flex items-center space-x-1.5">
      <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded border ${sourceColor}`}>{source}</span>
      <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded border ${statusColor}`}>{status}</span>
    </span>
  );
}

export function SkillSecurityInspector({ skill }: { skill: Record<string, unknown> | null }) {
  if (!skill) {
    return <div className="text-xs text-zinc-500">Select a skill to inspect trust, capabilities, and provenance.</div>;
  }
  const caps = (skill['capabilities_required'] as string[] | undefined) ?? [];
  const tools = (skill['allowed_tools'] as string[] | undefined) ?? [];
  const verification = (skill['verification_requirements'] as string[] | undefined) ?? [];
  return (
    <div className="space-y-3 text-xs">
      <div className="p-3 bg-zinc-950 rounded border border-zinc-800">
        <div className="font-semibold text-zinc-200 mb-1">Capability declaration (not authorization)</div>
        <p className="text-zinc-500">Skills declare capabilities. Actual grants remain authoritative in ToolRuntime / Filesystem / Network boundaries.</p>
        <div className="flex flex-wrap gap-1 mt-2">
          {caps.length === 0 && <span className="text-zinc-600">No capabilities declared</span>}
          {caps.map((c) => (
            <span key={c} className="font-mono px-1.5 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-zinc-300">{c}</span>
          ))}
        </div>
      </div>
      <div className="p-3 bg-zinc-950 rounded border border-zinc-800">
        <div className="font-semibold text-zinc-200 mb-1">Allowed tools</div>
        <div className="flex flex-wrap gap-1">
          {tools.length === 0 && <span className="text-zinc-600">None</span>}
          {tools.map((t) => (
            <span key={t} className="font-mono px-1.5 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-zinc-300">{t}</span>
          ))}
        </div>
      </div>
      <div className="p-3 bg-zinc-950 rounded border border-zinc-800">
        <div className="font-semibold text-zinc-200 mb-1">Verification requirements</div>
        <ul className="list-disc list-inside text-zinc-400">
          {verification.map((v) => (
            <li key={v} className="font-mono">{v}</li>
          ))}
        </ul>
      </div>
    </div>
  );
}

export function MemoryScopeBadge({ scope }: { scope: string }) {
  return (
    <span className="text-[10px] font-mono px-1.5 py-0.5 bg-violet-950/60 border border-violet-800/50 text-violet-300 rounded">
      {scope}
    </span>
  );
}

export function MemoryClassificationBadge({ classification }: { classification: PrivacyClassification }) {
  const color =
    classification === 'SECRET'
      ? 'bg-rose-950/60 border-rose-800/50 text-rose-300'
      : classification === 'RESTRICTED'
        ? 'bg-orange-950/60 border-orange-800/50 text-orange-300'
        : classification === 'CONFIDENTIAL'
          ? 'bg-amber-950/60 border-amber-800/50 text-amber-300'
          : 'bg-zinc-800 border-zinc-700 text-zinc-300';
  return (
    <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded border ${color}`}>{classification}</span>
  );
}
