'use client';

import React, { useEffect, useState } from 'react';
import { SkillSummary, SkillDefinition } from '@/lib/backend/types';
import { backendClient } from '@/lib/backend/client';
import { SkillSecurityBadge, SkillSecurityInspector } from '@/components/SkillMemoryBadges';

export default function SkillsPage() {
  const [skills, setSkills] = useState<SkillSummary[]>([]);
  const [selected, setSelected] = useState<SkillDefinition | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const fetchSkills = async () => {
    try {
      setLoading(true);
      setError(null);
      const res = await backendClient.listSkills();
      if (res.success) setSkills(res.skills);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Failed loading skills');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSkills();
  }, []);

  const loadDetails = async (id: string) => {
    try {
      const res = await backendClient.getSkill(id);
      if (res.success && res.skill) setSelected(res.skill);
    } catch (e: unknown) {
      setNotice(e instanceof Error ? e.message : 'Failed loading skill details');
    }
  };

  const act = async (id: string, op: 'enable' | 'disable' | 'validate' | 'approve') => {
    try {
      setNotice(null);
      if (op === 'enable') await backendClient.enableSkill(id);
      if (op === 'disable') await backendClient.disableSkill(id);
      if (op === 'validate') {
        const r = await backendClient.validateSkill(id);
        setNotice(r.valid ? 'Skill is VALID' : `Invalid: ${r.error ?? 'unknown'}`);
      }
      if (op === 'approve') await backendClient.approveSkill(id, 'ui-user');
      await fetchSkills();
      await loadDetails(id);
    } catch (e: unknown) {
      setNotice(e instanceof Error ? e.message : 'Action failed');
    }
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 p-8 space-y-6">
      <div className="border-b border-zinc-800 pb-5">
        <h1 className="text-2xl font-bold text-white">Skills Registry</h1>
        <p className="text-sm text-zinc-400 mt-1">
          Declarative, versioned, trust-ranked procedures. Declaration is never authorization — capabilities are enforced by ToolRuntime.
        </p>
      </div>
      {notice && <div className="p-3 bg-sky-950/40 border border-sky-800/50 rounded text-sky-300 text-xs">{notice}</div>}
      {loading ? (
        <div className="text-sm text-zinc-500">Loading skills…</div>
      ) : error ? (
        <div className="p-4 bg-red-950/40 border border-red-800/50 rounded text-red-300 text-sm">{error}</div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-7 bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
            <div className="px-5 py-3 border-b border-zinc-800 flex items-center justify-between">
              <h3 className="text-sm font-semibold">Registered Skills ({skills.length})</h3>
              <span className="text-xs text-zinc-500 font-mono">Fail-closed · versioned</span>
            </div>
            <div className="divide-y divide-zinc-800/60 max-h-[600px] overflow-y-auto">
              {skills.map((s) => (
                <div key={s.id} onClick={() => loadDetails(s.id)} className="p-4 hover:bg-zinc-800/30 cursor-pointer flex items-center justify-between">
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span className="font-semibold text-sm">{s.name}</span>
                      <span className="text-xs font-mono text-zinc-500">{s.versioned_id}</span>
                    </div>
                    <p className="text-xs text-zinc-400">{s.description}</p>
                    <SkillSecurityBadge source={s.source} status={s.status} />
                  </div>
                  <div className="flex items-center space-x-2">
                    <button onClick={(e) => { e.stopPropagation(); act(s.id, 'validate'); }} className="px-2 py-1 bg-zinc-800 rounded text-xs">Validate</button>
                    <button onClick={(e) => { e.stopPropagation(); act(s.id, s.status === 'ACTIVE' ? 'disable' : 'enable'); }} className="px-2 py-1 bg-zinc-800 rounded text-xs">
                      {s.status === 'ACTIVE' ? 'Disable' : 'Enable'}
                    </button>
                    {(s.source === 'IMPORTED' || s.source === 'MODEL_GENERATED' || s.source === 'EXTERNAL') && (
                      <button onClick={(e) => { e.stopPropagation(); act(s.id, 'approve'); }} className="px-2 py-1 bg-amber-900 rounded text-xs">Approve</button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
          <div className="lg:col-span-5 space-y-4">
            <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
              <h3 className="text-sm font-semibold mb-2">Skill Details</h3>
              {selected ? (
                <div className="space-y-2 text-xs">
                  <div className="font-mono text-zinc-500">{selected.id}@{selected.version}</div>
                  <SkillSecurityBadge source={selected.source} status={selected.status} />
                  <pre className="p-2 bg-zinc-950 border border-zinc-800 rounded text-[11px] overflow-auto">{JSON.stringify(selected, null, 2)}</pre>
                </div>
              ) : (
                <div className="text-xs text-zinc-500">Select a skill.</div>
              )}
            </div>
            <div className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
              <h3 className="text-sm font-semibold mb-2">Security Inspector</h3>
              <SkillSecurityInspector skill={selected as unknown as Record<string, unknown>} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
